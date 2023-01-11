pub struct ReportableProperty<Value> {
    value: Value,
    modified: bool,
}

impl<Value: PartialEq + Copy> ReportableProperty<Value> {
    pub fn new(value: Value) -> ReportableProperty<Value> {
        ReportableProperty {
            value,
            modified: false,
        }
    }

    pub fn get_value(&self) -> Value {
        self.value
    }

    pub fn set_value(&mut self, value: Value, force: bool) {
        if force || self.value != value {
            self.value = value;
            self.modified = true;
        }
    }

    pub fn modified(&self) -> bool {
        self.modified
    }

    pub fn reset_modified(&mut self) {
        self.modified = false;
    }
}

impl<Value: PartialEq> PartialEq for ReportableProperty<Value> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<Value: Default> Default for ReportableProperty<Value> {
    fn default() -> Self {
        ReportableProperty {
            value: Value::default(),
            modified: false,
        }
    }
}
