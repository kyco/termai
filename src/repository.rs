pub trait Repository {
    type Error;

    fn get_messages(&self) -> Result<Vec<String>, Self::Error>;
    fn add_message(&self, content: String) -> Result<(), Self::Error>;
}
