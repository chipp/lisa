use transport::elisa::Action as ElisaAction;
use transport::elizabeth::Action as ElizabethAction;

#[derive(Debug, PartialEq)]
pub enum Action {
    Elizabeth(ElizabethAction),
    Elisa(ElisaAction),
}

impl From<ElisaAction> for Action {
    fn from(value: ElisaAction) -> Self {
        Self::Elisa(value)
    }
}

impl From<ElizabethAction> for Action {
    fn from(value: ElizabethAction) -> Self {
        Self::Elizabeth(value)
    }
}
