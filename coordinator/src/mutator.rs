use rand::Rng;
use ciborium::Value;

fn mutate_ast(value: &mut Value, rng: &mut impl Rng) {
    match value {
        Value::Float(f) => {
            if rng.gen_bool(0.5) {
                *f = (*f as f32) as f64; 
            }
        },
        Value::Bytes(b) => {
            if rng.gen_bool(0.3) {
                let tag_num = if rng.gen_bool(0.5) { 42 } else { rng.gen_range(1..100) };
                *value = Value::Tag(tag_num, Box::new(Value::Bytes(b.clone())));
            }
        },
        Value::Array(arr) => {
            for item in arr.iter_mut() { mutate_ast(item, rng); }
        },
        Value::Map(map) => {
            if rng.gen_bool(0.2) {
                map.push((Value::Integer(rng.gen_range(1..100).into()), Value::Text("corrupted".to_string())));
            }
            if rng.gen_bool(0.1) && !map.is_empty() {
                let clone = map[0].clone();
                map.push(clone);
            }
            for (k, v) in map.iter_mut() {
                mutate_ast(k, rng);
                mutate_ast(v, rng);
            }
        },
        Value::Tag(_, inner) => mutate_ast(inner, rng),
        _ => {}
    }
}

pub fn generate_mutant(seed_bytes: &[u8], rng: &mut impl Rng) -> Option<Vec<u8>> {
    let mut ast: Value = ciborium::from_reader(seed_bytes).ok()?;
    mutate_ast(&mut ast, rng);

    let mut mutated_bytes = Vec::new();
    ciborium::into_writer(&ast, &mut mutated_bytes).unwrap();

    if rng.gen_bool(0.15) {
        mutated_bytes.push(rng.gen());
    }

    Some(mutated_bytes)
}