#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Operation {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "foxy")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub operation: Operation,
    pub id: String,
    pub time: f64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let notification_str = include_str!("../tests/data/notification.json");
        let notification: Result<Notification, _> = serde_json::from_str(notification_str);
        assert!(notification.is_ok());
        let notification = notification.unwrap();
        assert_eq!(notification.operation, Operation::Update);
    }
}
