use super::pending_closes::PendingCloseBook;
use super::pending_entries::PendingEntryBook;
use super::pending_exits::PendingExitBook;
use super::types::{InternalOrderKey, OcaGroupKey, OcaMember, OcaPeerEffects, OcaType};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct OrderBook {
    entries: PendingEntryBook,
    exits: PendingExitBook,
    closes: PendingCloseBook,
    oca_membership: HashMap<OcaMember, OcaGroupKey>,
}

impl OrderBook {
    pub(super) fn new() -> Self {
        Self {
            entries: PendingEntryBook::new(),
            exits: PendingExitBook::new(),
            closes: PendingCloseBook::new(),
            oca_membership: HashMap::new(),
        }
    }

    pub(super) fn entries(&self) -> &PendingEntryBook {
        &self.entries
    }

    pub(super) fn entries_mut(&mut self) -> &mut PendingEntryBook {
        &mut self.entries
    }

    pub(super) fn exits(&self) -> &PendingExitBook {
        &self.exits
    }

    pub(super) fn exits_mut(&mut self) -> &mut PendingExitBook {
        &mut self.exits
    }

    #[allow(dead_code)]
    pub(super) fn closes(&self) -> &PendingCloseBook {
        &self.closes
    }

    pub(super) fn closes_mut(&mut self) -> &mut PendingCloseBook {
        &mut self.closes
    }

    pub(super) fn cancel_id(&mut self, id: &str) {
        let members = self.pending_members_for_public_id(id);
        self.entries.cancel_id(id);
        self.exits.cancel_id(id);
        self.closes.cancel_id(id);
        for member in members {
            self.oca_membership.remove(&member);
        }
        self.prune_oca_membership();
    }

    pub(super) fn clear_all(&mut self) {
        self.entries.clear_all();
        self.exits.clear_all();
        self.closes.clear_all();
        self.oca_membership.clear();
    }

    pub(super) fn clear_exits_for_entry(&mut self, entry_id: &str) {
        self.exits.clear_for_entry(entry_id);
        self.prune_oca_membership();
    }

    #[allow(dead_code)]
    pub(super) fn assign_oca(&mut self, member: OcaMember, group: OcaGroupKey) {
        self.oca_membership.insert(member, group);
    }

    #[allow(dead_code)]
    pub(super) fn oca_group(&self, member: &OcaMember) -> Option<&OcaGroupKey> {
        self.oca_membership.get(member)
    }

    #[allow(dead_code)]
    pub(super) fn oca_members_in_group(&self, group: &OcaGroupKey) -> Vec<OcaMember> {
        let mut members = self
            .oca_membership
            .iter()
            .filter(|(_, assigned)| *assigned == group)
            .map(|(member, _)| member.clone())
            .collect::<Vec<_>>();
        members.sort_by_key(|member| match member {
            OcaMember::Order(key) => (0, key.0),
            OcaMember::Exit {
                target_trade_key, ..
            } => (1, target_trade_key.unwrap_or(u64::MAX)),
        });
        members
    }

    pub(super) fn apply_oca_after_exit_fill(
        &mut self,
        id: &str,
        from_entry: &str,
        target_trade_key: Option<u64>,
        filled_qty: f64,
    ) {
        let filled = OcaMember::Exit {
            id: id.to_owned(),
            from_entry: from_entry.to_owned(),
            target_trade_key,
        };
        let Some(group) = self.oca_membership.get(&filled).cloned() else {
            return;
        };
        self.oca_membership.remove(&filled);
        if group.oca_type != OcaType::Reduce {
            return;
        }
        let peers = self.oca_members_in_group(&group);
        for peer in peers {
            let OcaMember::Exit {
                id: peer_id,
                from_entry: peer_from,
                target_trade_key: peer_key,
            } = peer
            else {
                continue;
            };
            if peer_id == id && peer_from == from_entry && peer_key == target_trade_key {
                continue;
            }
            if let Some(pending) = self
                .exits
                .find_mut_by_identity_and_key(&peer_id, &peer_from, peer_key)
            {
                let remaining = (pending.reserved_quantity - filled_qty).max(0.0);
                if remaining <= 0.0 {
                    self.exits
                        .remove_by_identity_and_key(&peer_id, &peer_from, peer_key);
                    self.oca_membership.remove(&OcaMember::Exit {
                        id: peer_id,
                        from_entry: peer_from,
                        target_trade_key: peer_key,
                    });
                } else {
                    pending.reserved_quantity = remaining;
                }
            } else {
                self.oca_membership.remove(&OcaMember::Exit {
                    id: peer_id,
                    from_entry: peer_from,
                    target_trade_key: peer_key,
                });
            }
        }
    }

    pub(super) fn apply_oca_after_fill(
        &mut self,
        filled_key: InternalOrderKey,
        filled_qty: f64,
    ) -> OcaPeerEffects {
        let filled = OcaMember::Order(filled_key);
        let Some(group) = self.oca_membership.get(&filled).cloned() else {
            return OcaPeerEffects::default();
        };
        self.oca_membership.remove(&filled);
        match group.oca_type {
            OcaType::None => OcaPeerEffects::default(),
            OcaType::Cancel => self.reduce_or_cancel_oca_peers(filled_key, &group, None),
            OcaType::Reduce => {
                self.reduce_or_cancel_oca_peers(filled_key, &group, Some(filled_qty))
            }
        }
    }

    fn reduce_or_cancel_oca_peers(
        &mut self,
        filled_key: InternalOrderKey,
        group: &OcaGroupKey,
        reduce_by: Option<f64>,
    ) -> OcaPeerEffects {
        let peers = self.oca_members_in_group(group);
        let mut effects = OcaPeerEffects::default();
        for peer in peers {
            let OcaMember::Order(key) = peer else {
                continue;
            };
            if key == filled_key {
                continue;
            }
            match reduce_by {
                None => {
                    self.entries.remove_by_key(key);
                    self.oca_membership.remove(&OcaMember::Order(key));
                    effects.cancelled.push(key);
                }
                Some(filled_qty) => {
                    if let Some(pending) = self.entries.find_mut_by_key(key) {
                        let remaining = (pending.quantity - filled_qty).max(0.0);
                        if remaining <= 0.0 {
                            self.entries.remove_by_key(key);
                            self.oca_membership.remove(&OcaMember::Order(key));
                            effects.cancelled.push(key);
                            effects.reduced.insert(key, 0.0);
                        } else {
                            pending.quantity = remaining;
                            effects.reduced.insert(key, remaining);
                        }
                    } else {
                        effects.reduce_taken.push(key);
                    }
                }
            }
        }
        effects
    }

    pub(super) fn clear_oca_order(&mut self, key: InternalOrderKey) {
        self.oca_membership.remove(&OcaMember::Order(key));
    }

    fn prune_oca_membership(&mut self) {
        let live_order_keys = self
            .entries
            .iter()
            .map(|pending| pending.key)
            .chain(self.closes.iter().map(|pending| pending.key))
            .collect::<HashSet<_>>();
        self.oca_membership.retain(|member, _| match member {
            OcaMember::Order(key) => live_order_keys.contains(key),
            OcaMember::Exit {
                id,
                from_entry,
                target_trade_key,
            } => self
                .exits
                .find_by_identity_and_key(id, from_entry, *target_trade_key)
                .is_some(),
        });
    }

    fn pending_members_for_public_id(&self, id: &str) -> Vec<OcaMember> {
        let mut members = Vec::new();
        for pending in self.entries.iter() {
            if pending.id == id {
                members.push(OcaMember::Order(pending.key));
            }
        }
        for pending in self.exits.iter() {
            if pending.id == id {
                members.push(OcaMember::Exit {
                    id: pending.id.clone(),
                    from_entry: pending.from_entry.clone(),
                    target_trade_key: pending.target_trade_key,
                });
            }
        }
        for pending in self.closes.iter() {
            if pending.public_id() == Some(id) {
                members.push(OcaMember::Order(pending.key));
            }
        }
        members
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
