impl Ast {
	pub fn build_dfa(&self) -> DFA<usize> {
		let nd = self.build_nfa();

		let mut map = HashMap::new();
		let mut n = 0usize;
		let dt = nd.determinize(|q| {
			*map.entry(q.clone()).or_insert_with(|| {
				let i = n;
				n += 1;
				i
			})
		});
		debug_assert!(!dt.final_states().is_empty());
		dt
	}

	pub fn build_nfa(&self) -> NFA<usize> {
		let mut result = NFA::new();

		let mut n = 0;
		let mut new_state = move || {
			let r = n;
			n += 1;
			r
		};

		let (a, b) = self.build_into(&mut new_state, &mut result);
		result.add_initial_state(a);
		result.add_final_state(b);
		debug_assert!(!result.final_states().is_empty());

		result
	}

	fn build_into(
		&self,
		new_state: &mut impl FnMut() -> usize,
		automaton: &mut NFA<usize>,
	) -> (usize, usize) {
		match self {
			Self::Any => {
				let mut charset = RangeSet::new();
				charset.insert('\u{0}'..='\u{d7ff}');
				charset.insert('\u{e000}'..='\u{10ffff}');
				let a = new_state();
				let b = new_state();
				automaton.add(a, Some(charset), b);
				(a, b)
			}
			Self::Repeat(exp, min, max) => exp.build_repeat_into(new_state, automaton, *min, *max),
			Self::Sequence(exps) => {
				let a = new_state();
				let mut b = a;

				for e in exps {
					let (ea, eb) = e.build_into(new_state, automaton);
					automaton.add(b, None, ea);
					b = eb;
				}

				(a, b)
			}
			Self::Set(charset) => {
				let a = new_state();
				let b = new_state();

				automaton.add(a, Some(charset.clone()), b);
				(a, b)
			}
			Self::Union(exps) => {
				let a = new_state();
				let b = new_state();

				for e in exps {
					let (ea, eb) = e.build_into(new_state, automaton);
					automaton.add(a, None, ea);
					automaton.add(eb, None, b);
				}

				(a, b)
			}
		}
	}

	fn build_repeat_into(
		&self,
		new_state: &mut impl FnMut() -> usize,
		automaton: &mut NFA<usize>,
		min: u32,
		max: u32,
	) -> (usize, usize) {
		if max == 0 {
			let a = new_state();
			(a, a)
		} else if min > 0 {
			let (a, b) = self.build_into(new_state, automaton);
			let (rest_a, rest_b) = self.build_repeat_into(
				new_state,
				automaton,
				min - 1,
				if max < u32::MAX { max - 1 } else { u32::MAX },
			);
			automaton.add(b, None, rest_a);
			(a, rest_b)
		} else if max < u32::MAX {
			let (a, b) = self.build_into(new_state, automaton);
			let (c, d) = self.build_repeat_into(new_state, automaton, 0, max - 1);
			automaton.add(a, None, d);
			automaton.add(b, None, c);
			(a, d)
		} else {
			let (a, b) = self.build_into(new_state, automaton);
			automaton.add(a, None, b);
			automaton.add(b, None, a);
			(a, b)
		}
	}
}