use crate::search_for_locations::ExternalLocationId;
use uuid::Uuid;

pub struct LocationInput<T: Clone> {
    pub id: T,
    pub nearby_locations: Vec<T>,
}

impl<T: Clone> LocationInput<T> {
    pub fn primary_id(&self) -> &T {
        &self.id
    }
    pub fn ids(&self) -> Vec<T> {
        let mut ids = self.nearby_locations.to_vec();
        ids.push(self.id.clone());
        ids
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LocationId(Uuid);

pub struct LocationWithId {
    pub id: LocationId,
}
