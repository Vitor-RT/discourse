use super::collection::{Collection, CollectionItem, Item};

#[derive(Debug, Clone)]
pub struct Field {
    pub item: Item,
    pub field_type: String,
}

impl Field {
    pub fn new(id: String, field_type: String) -> Self {
        Self {
            item: Item::new(id),
            field_type,
        }
    }
}

impl CollectionItem for Field {
    fn server_action(&self) -> String {
        self.item.id.clone()
    }
}

pub type EditableFields<R, G, A> = Collection<Field, R, G, A>;

impl<R, G, A> EditableFields<R, G, A> {
    pub fn add(&mut self, id: String, field_type: String) {
        self.content.push(Field::new(id, field_type));
    }
}
