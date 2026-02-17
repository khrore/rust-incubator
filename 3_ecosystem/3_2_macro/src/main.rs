macro_rules! btreemap {
    () => {{
        ::std::collections::BTreeMap::new()
    }};
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::BTreeMap::new();
        $(
            map.insert($key, $value);
        )+
        map
    }};
}

fn main() {
    let declarative = btreemap! {
        3 => "three",
        1 => "one",
        2 => "two",
    };
    let procedural = step_3_2_proc::btreemap! {
        "alice" => 10_u32,
        "bob" => 20_u32,
    };

    println!("Declarative btreemap!: {:?}", declarative);
    println!("Procedural btreemap!: {:?}", procedural);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn declarative_macro_creates_empty_map() {
        let map: BTreeMap<u32, u32> = btreemap! {};
        assert!(map.is_empty());
    }

    #[test]
    fn declarative_macro_creates_sorted_map() {
        let map = btreemap! {
            3 => "three",
            1 => "one",
            2 => "two",
        };

        let keys = map.keys().copied().collect::<Vec<_>>();
        assert_eq!(keys, vec![1, 2, 3]);
    }

    #[test]
    fn procedural_macro_creates_empty_map() {
        let map: BTreeMap<u32, u32> = step_3_2_proc::btreemap! {};
        assert!(map.is_empty());
    }

    #[test]
    fn procedural_macro_creates_map() {
        let map = step_3_2_proc::btreemap! {
            "alice" => 10_u32,
            "bob" => 20_u32,
        };

        assert_eq!(map.get("alice"), Some(&10));
        assert_eq!(map.get("bob"), Some(&20));
    }

    #[test]
    fn both_implementations_match() {
        let declarative = btreemap! {
            1 => "one",
            2 => "two",
        };
        let procedural = step_3_2_proc::btreemap! {
            1 => "one",
            2 => "two",
        };

        assert_eq!(declarative, procedural);
    }
}
