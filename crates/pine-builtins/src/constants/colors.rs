pub struct NamedColor {
    pub name: &'static str,
    pub rgb: u32,
}

pub const NAMED_COLORS: &[NamedColor] = &[
    NamedColor {
        name: "color.black",
        rgb: 0x000000,
    },
    NamedColor {
        name: "color.silver",
        rgb: 0xC0C0C0,
    },
    NamedColor {
        name: "color.gray",
        rgb: 0x808080,
    },
    NamedColor {
        name: "color.white",
        rgb: 0xFFFFFF,
    },
    NamedColor {
        name: "color.maroon",
        rgb: 0x800000,
    },
    NamedColor {
        name: "color.red",
        rgb: 0xFF0000,
    },
    NamedColor {
        name: "color.purple",
        rgb: 0x800080,
    },
    NamedColor {
        name: "color.fuchsia",
        rgb: 0xFF00FF,
    },
    NamedColor {
        name: "color.green",
        rgb: 0x008000,
    },
    NamedColor {
        name: "color.lime",
        rgb: 0x00FF00,
    },
    NamedColor {
        name: "color.olive",
        rgb: 0x808000,
    },
    NamedColor {
        name: "color.yellow",
        rgb: 0xFFFF00,
    },
    NamedColor {
        name: "color.navy",
        rgb: 0x000080,
    },
    NamedColor {
        name: "color.blue",
        rgb: 0x0000FF,
    },
    NamedColor {
        name: "color.teal",
        rgb: 0x008080,
    },
    NamedColor {
        name: "color.aqua",
        rgb: 0x00FFFF,
    },
    NamedColor {
        name: "color.orange",
        rgb: 0xFF9900,
    },
];

#[must_use]
pub fn named_color(name: &str) -> Option<u32> {
    NAMED_COLORS
        .iter()
        .find(|color| color.name == name)
        .map(|color| color.rgb)
}
