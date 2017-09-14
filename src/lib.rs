
pub struct Runtime {
	instructions: String,
	instruction_pointer: usize,

	input: Vec<u8>,
	input_pointer: usize,

	memory: Vec<u8>,
	memory_pointer: usize,

	output: Vec<u8>
}

pub struct RuntimeSnapshot {
	pub memory: Vec<u8>,
	pub memory_pointer: usize,
	pub instruction_pointer: usize,
	pub input_pointer: usize,
	pub output: Vec<u8>,

	pub is_error: bool,
	pub message: &'static str
}

impl RuntimeSnapshot {

	fn new(runtime: &Runtime, is_error: bool, message: &'static str) -> RuntimeSnapshot {
		RuntimeSnapshot {
			memory: runtime.memory.to_vec(),
			memory_pointer: runtime.memory_pointer,
			instruction_pointer: runtime.instruction_pointer,
			input_pointer: runtime.input_pointer,
			output: runtime.output.to_vec(),

			is_error: is_error,
			message: message
		}
	}

}

pub struct RuntimeProduct {
	pub snapshots: Vec<RuntimeSnapshot>,
	pub output: Vec<u8>
}

type RuntimeResult = Result<&'static str, &'static str>;

impl Runtime {

	pub fn new(instructions: String, input: Vec<u8>) -> Runtime {
		Runtime {
			instructions: instructions,
			instruction_pointer: 0,

			input: input,
			input_pointer: 0,

			memory: vec![0; 1],
			memory_pointer: 0,

			output: Vec::new()
		}
	}

	fn expand_memory(&mut self) {
		let additional = self.memory.capacity() / 2; // reserve 50% of the current capacity more
		self.memory.reserve_exact(additional);

		// TODO assert: self.memory_pointer == memory.capacity()
		self.memory.extend(vec![0; additional]);
	}

	fn next_input_byte(&mut self) -> u8 {
		if self.input_pointer >= self.input.len() {
			return 255; // TODO: -1?
		}

		let result = self.input[self.input_pointer];
		self.input_pointer += 1;
		result
	}

	fn increment_pointer(&mut self) -> RuntimeResult {
		// ensure capacity
		if (self.memory_pointer + 1) >= self.memory.capacity() {
			// TODO: memory limit check?
			self.expand_memory();
		}

		self.memory_pointer += 1; // increment the pointer
		Ok("incremented pointer by 1")
	}

	fn decrement_pointer(&mut self) -> RuntimeResult {
		if self.memory_pointer <= 0 { // can't decrement to below zero
			return Err("can't decrement pointer sub-0!");
		}

		self.memory_pointer -= 1;
		Ok("decremented pointer by 1")
	}

	fn increment_byte(&mut self) -> RuntimeResult {
		if self.memory[self.memory_pointer] < 255 {
			self.memory[self.memory_pointer] += 1;
			return Ok("incremented byte by 1");
		} else {
			self.memory[self.memory_pointer] = 0;
			return Ok("wrapped overflow byte back to 0x00");
		}
	}

	fn decrement_byte(&mut self) -> RuntimeResult {
		if self.memory[self.memory_pointer] > 0 {
			self.memory[self.memory_pointer] -= 1;
			return Ok("decremented byte by 1");
		} else {
			self.memory[self.memory_pointer] = 255;
			return Ok("wrapped overflow byte back to 0xFF");
		}
	}

	fn output_byte(&mut self) -> RuntimeResult {
		// TODO: output length check?
		let this_byte = self.memory[self.memory_pointer];
		self.output.push(this_byte);

		Ok("copied byte from memory to output")
	}

	fn input_byte(&mut self) -> RuntimeResult {
		self.memory[self.memory_pointer] = self.next_input_byte();
		Ok("copied byte from input to memory")
	}

	fn handle_open_bracket(&mut self) -> RuntimeResult {
		if self.memory[self.memory_pointer] == 0 {

			let mut open_count: u16 = 0;
			loop {
				if (self.instruction_pointer + 1) >= self.instructions.len() {
					return Err("hit end of instructions w/o finding matching close bracket!");
				}

				self.instruction_pointer += 1;
				match self.instructions.chars().nth(self.instruction_pointer).unwrap() {
					'[' => open_count += 1,
					']' => {
						if open_count > 0 { // if there are open brackets left closed
							open_count -= 1;
						} else { // else, there are no open brackets left to consume
							return Ok("found matching close bracket");
						}
					},
					_ => ()
				}
			}
		} else {
			return Ok("byte is non-zero, no bracket seek necessary");
		}
	}

	fn handle_close_bracket(&mut self) -> RuntimeResult {
		if self.memory[self.memory_pointer] != 0 {

			let mut close_count: u16 = 0;
			loop {
				if self.instruction_pointer <= 0 {
					return Err("hit beginning of instructions w/o finding matching open bracket!")
				}

				self.instruction_pointer -= 1;
				match self.instructions.chars().nth(self.instruction_pointer).unwrap() {
					']' => close_count += 1,
					'[' => {
						if close_count > 0 { // if there are closed brackets left open
							close_count -= 1;
						} else {
							return Ok("found matching open bracket");
						}
					},
					_ => ()
				}
			}
		} else {
			return Ok("byte is zero, no bracket seek necessary");
		}
	}

	pub fn run(&mut self) -> RuntimeProduct {
		let mut snapshots: Vec<RuntimeSnapshot> = Vec::new();

		while self.instruction_pointer < self.instructions.len() {

			let mut result: Option<RuntimeResult> = None;
			match self.instructions.chars().nth(self.instruction_pointer).unwrap() {
				'>' => result = Some(self.increment_pointer()),
				'<' => result = Some(self.decrement_pointer()),
				'+' => result = Some(self.increment_byte()),
				'-' => result = Some(self.decrement_byte()),
				'.' => result = Some(self.output_byte()),
				',' => result = Some(self.input_byte()),
				'[' => result = Some(self.handle_open_bracket()),
				']' => result = Some(self.handle_close_bracket()),
				_ => ()
			}

			if let Some(runtime_result) = result {
				if runtime_result.is_ok() {
					snapshots.push(RuntimeSnapshot::new(&self, false, runtime_result.ok().unwrap()));
				} else {
					snapshots.push(RuntimeSnapshot::new(&self, true, runtime_result.err().unwrap()));
					break; // all errors are fatal
				}
			}

			self.instruction_pointer += 1;
		}

		RuntimeProduct {
			snapshots: snapshots,
			output: self.output.clone()
		}
	}

}
