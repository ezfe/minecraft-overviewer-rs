pub fn is_complex_geometry(name: &str) -> bool {
    name.contains("litter")
        || name.contains("sapling")
        || name.contains("flower")
        || name.contains("grass") && !name.contains("block")
        || name.contains("fern")
        || name.contains("dead_bush")
        || name.contains("seagrass")
        || name.contains("kelp")
        || name.contains("vine")
        || name.contains("lily_pad")
        || name.contains("torch")
        || name.contains("fire")
        || name.contains("redstone_wire")
        || name.contains("rail")
        || name.contains("ladder")
        || name.contains("lever")
        || name.contains("button")
        || name.contains("pressure_plate")
        || name.contains("tripwire")
        || name.contains("string")
        || name.contains("carpet") && !name.contains("moss")
        || name.contains("fence") && !name.contains("gate")
        || name.contains("wall") && !name.contains("sign")
        || name.contains("bars")
        || name.contains("chain")
        || name.contains("lantern")
        || name.contains("candle")
        || name.contains("rod")
        || name.contains("banner")
        || name.contains("sign")
        || name.contains("head")
        || name.contains("skull")
        || name.contains("dripstone")
        || name.contains("pointed")
        || name.contains("amethyst_cluster")
        || name.contains("amethyst_bud")
}

pub fn is_air_block(name: &str) -> bool {
    name == "minecraft:air" || name == "minecraft:cave_air" || name == "minecraft:void_air"
}

pub fn is_solid_block(name: &str) -> bool {
    !is_complex_geometry(name) && !is_air_block(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_is_not_solid() {
        assert_eq!(is_solid_block("minecraft:air"), false, "Air should not be solid");
    }
}
