use phf::phf_map;

static INSTRUCTION_TYPES: phf::Map<&str, u8> = phf_map!(
    "UNION" => 255,
    "INTER" => 254,
    "SUBST" => 253,

    "SPHERE" => 1,
    "BOX" => 2,
);

pub fn u32_to_u8s(n: u32) -> [u8; 4] {
    [
        (n >> 24) as u8,
        ((n >> 16) & 255u32) as u8,
        ((n >> 8) & 255u32) as u8,
        (n & 255u32) as u8,
    ]
}

pub fn f32_to_u8s(n: f32) -> [u8; 4] {
    u32_to_u8s(n.to_bits())
}

pub fn build_instruction_bytes(instructions_str: &str) -> Vec<u8> {
    let mut instructions_str = instructions_str.to_string();
    instructions_str = instructions_str.replace(':', " ");
    instructions_str = instructions_str.replace(',', " ");
    instructions_str = instructions_str.replace('(', " ");
    instructions_str = instructions_str.replace(')', " ");

    let instructions = instructions_str
        .split_whitespace()
        .map(|i| {
            let if32 = i.parse::<f32>();
            match if32 {
                Ok(if32) => f32_to_u8s(if32).to_vec(),
                Err(_) => vec![INSTRUCTION_TYPES.get(i).unwrap().clone()],
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    instructions
}
