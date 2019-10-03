use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/markdown_entities.rs"));

fn get_map() -> &'static HashMap<&'static str, &'static str> {
	&ENTITIES
}

pub fn get_named_entity(entity: &str) -> Option<&'static str> {
	get_map().get(entity).map(|x| *x)
}
