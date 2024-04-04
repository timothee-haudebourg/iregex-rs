use iregex_automata::Class;

pub trait Boundary<T> {
	type Class: Class<T>;

	fn apply(&self, class: &Self::Class) -> Option<Self::Class>;
}

impl<T> Boundary<T> for () {
	type Class = ();

	fn apply(&self, _class: &Self::Class) -> Option<Self::Class> {
		Some(())
	}
}
