/// Represents a parameter field in Open Protocol data section
#[derive(Debug, Clone)]
pub struct Field {
    /// Parameter ID (e.g., "01", "02", etc.)
    pub id: Option<String>,
    /// Parameter value
    pub value: String,
}

impl Field {
    /// Create a new field with a parameter ID and value
    #[allow(dead_code)]
    pub fn new(id: Option<u8>, value: impl Into<String>) -> Self {
        let id = id.map(|v| format!("{:02}", v));
        Self {
            id,
            value: value.into(),
        }
    }

    /// Create a field from an integer value
    pub fn from_int(id: Option<u8>, value: i32, width: usize) -> Self {
        let id = id.map(|v| format!("{:02}", v));
        Self {
            id,
            value: format!("{:0width$}", value, width = width),
        }
    }

    /// Create a field from a float value
    pub fn from_float(id: Option<u8>, value: f64, width: usize, precision: usize) -> Self {
        let id = id.map(|v| format!("{:02}", v));
        Self {
            id,
            value: format!("{:0width$.precision$}", value, width = width, precision = precision),
        }
    }

    /// Create a field from a string value with fixed width (space-padded)
    pub fn from_str(id: Option<u8>, value: impl AsRef<str>, width: usize) -> Self {
        let s = value.as_ref();
        let padded = if s.len() >= width {
            s[..width].to_string()
        } else {
            format!("{:<width$}", s, width = width)
        };
        let id = id.map(|v| format!("{:02}", v));
        Self {
            id,
            value: padded,
        }
    }

    /// Serialize this field in Open Protocol format
    /// Format: [ParamID][Value]
    /// Note: Some Open Protocol versions include a length field, but this implementation
    /// uses direct field serialization
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        if let Some(id) = &self.id {
            result.extend_from_slice(id.as_bytes());
        }

        result.extend_from_slice(self.value.as_bytes());

        result
    }
}

/// Builder for constructing parameter fields
pub struct FieldBuilder {
    fields: Vec<Field>,
}

impl FieldBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub fn add_int(self, id: Option<u8>, value: i32, width: usize) -> Self {
        self.add_field(Field::from_int(id, value, width))
    }

    pub fn add_float(self, id: Option<u8>, value: f64, width: usize, precision: usize) -> Self {
        self.add_field(Field::from_float(id, value, width, precision))
    }

    pub fn add_str(self, id: Option<u8>, value: impl AsRef<str>, width: usize) -> Self {
        self.add_field(Field::from_str(id, value, width))
    }

    pub fn build(self) -> Vec<u8> {
        let mut result = Vec::new();
        for field in self.fields {
            result.extend_from_slice(&field.serialize());
        }
        result
    }
}

impl Default for FieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_serialization() {
        let field = Field::new(Some(1), "TEST");
        let serialized = field.serialize();
        // Format: 01 04 TEST
        // "01" (param ID) + "04" (length) + "TEST" (value)
        assert_eq!(serialized, b"01TEST");
    }

    #[test]
    fn test_int_field() {
        let field = Field::from_int(Some(2), 123, 5);
        let serialized = field.serialize();
        assert_eq!(serialized, b"0200123");
    }

    #[test]
    fn test_str_field() {
        let field = Field::from_str(Some(3), "ABC", 10);
        let serialized = field.serialize();
        assert_eq!(serialized, b"03ABC       ");
    }

    #[test]
    fn test_builder() {
        let data = FieldBuilder::new()
            .add_int(Some(1), 100, 3)
            .add_str(Some(2), "TEST", 5)
            .build();

        // Verify the data contains both fields
        assert!(!data.is_empty());

        // Convert to string for easier debugging
        let data_str = String::from_utf8_lossy(&data);

        // Should start with field 1: param id "01"
        assert!(data_str.starts_with("01"));

        // Should contain field 2: param id "02"
        assert!(data_str.contains("02"));

        // Should contain the test string
        assert!(data_str.contains("TEST"));
    }
}
