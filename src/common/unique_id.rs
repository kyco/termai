use uuid::Uuid;

pub fn generate_uuid_v4() -> Uuid {
    Uuid::new_v4()
}
