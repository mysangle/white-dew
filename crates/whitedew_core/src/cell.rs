
// RefCell: 컴파일 타임 borrow 규칙을 런타임에 검사
pub type SemiRefCell<T> = std::cell::RefCell<T>;
// Ref: 불변 참조를 표현하는 borrow guard
pub type Ref<'b, T> = std::cell::Ref<'b, T>;
// RefMut: 가변 참조를 표현하는 mutable borrow guard
pub type RefMut<'b, T> = std::cell::RefMut<'b, T>;
