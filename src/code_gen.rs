use std::{ffi::OsStr, mem::size_of, path::Path};

use crate::instruction::Instruction;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{BasicMetadataTypeEnum, BasicType, IntType, PointerType, VoidType},
    values::{FunctionValue, GlobalValue, PointerValue},
    AddressSpace, IntPredicate,
};

struct Types<'a> {
    void_t: VoidType<'a>,
    bool_t: IntType<'a>,
    char_t: IntType<'a>,
    char_ptr_t: PointerType<'a>,
    char_ptr_ptr_t: PointerType<'a>,
    int_t: IntType<'a>,
    size_t_t: IntType<'a>,
    size_t_ptr_t: PointerType<'a>,
    file_ptr_t: PointerType<'a>,
}

impl<'a> Types<'a> {
    fn new(context: &'a Context) -> Self {
        let addr_space = AddressSpace::default();

        let void_t = context.void_type();
        let bool_t = context.bool_type();
        let char_t = Self::get_int_type::<libc::c_char>(context);
        let char_ptr_t = char_t.ptr_type(addr_space);
        let char_ptr_ptr_t = char_ptr_t.ptr_type(addr_space);
        let int_t = Self::get_int_type::<libc::c_int>(context);
        let size_t_t = Self::get_int_type::<libc::size_t>(context);
        let size_t_ptr_t = size_t_t.ptr_type(addr_space);

        let file_ptr_t = context.opaque_struct_type("__sFILE").ptr_type(addr_space);

        Self {
            void_t,
            bool_t,
            char_t,
            char_ptr_t,
            char_ptr_ptr_t,
            int_t,
            size_t_t,
            size_t_ptr_t,
            file_ptr_t,
        }
    }

    fn get_int_type<T>(context: &'a Context) -> IntType<'a> {
        match size_of::<T>() {
            1 => context.i8_type(),
            2 => context.i16_type(),
            4 => context.i32_type(),
            8 => context.i64_type(),
            _ => panic!("Unsupported integer size: {}", size_of::<T>()),
        }
    }
}

struct Globals<'a> {
    stdout_ptr_v: GlobalValue<'a>,
    stderr_ptr_v: GlobalValue<'a>,
    error_string_v: GlobalValue<'a>,
}

impl<'a> Globals<'a> {
    fn new(context: &'a Context, module: &Module<'a>, types: &Types<'a>) -> Self {
        let stdout_ptr_v = module.add_global(types.file_ptr_t, None, "__stdoutp");
        stdout_ptr_v.set_alignment(8);
        let stderr_ptr_v = module.add_global(types.file_ptr_t, None, "__stderrp");
        stderr_ptr_v.set_alignment(8);

        let error_string_v = Self::create_string(
            "Error: Cannot move pointer to negative cell!\n",
            "errorString",
            context,
            module,
        );

        Self {
            stdout_ptr_v,
            stderr_ptr_v,
            error_string_v,
        }
    }

    fn create_string<'b>(
        value: &str,
        name: &str,
        context: &'b Context,
        module: &Module<'b>,
    ) -> GlobalValue<'b> {
        let string_constant = context.const_string(value.as_bytes(), true);
        let string_v = module.add_global(string_constant.get_type(), None, name);
        string_v.set_constant(true);
        string_v.set_linkage(Linkage::Private);
        string_v.set_initializer(&string_constant);
        string_v.set_unnamed_addr(true);
        string_v.set_alignment(1);

        string_v
    }
}

struct Functions<'a> {
    calloc_f: FunctionValue<'a>,
    free_f: FunctionValue<'a>,
    fputs_f: FunctionValue<'a>,
    putchar_f: FunctionValue<'a>,
    fflush_f: FunctionValue<'a>,
    move_right_f: FunctionValue<'a>,
    input_f: FunctionValue<'a>,
    move_right_until_zero_f: FunctionValue<'a>,
    move_left_until_zero_f: FunctionValue<'a>,
    move_value_right_f: FunctionValue<'a>,
    move_value_left_f: FunctionValue<'a>,
    main_f: FunctionValue<'a>,
}

impl<'a> Functions<'a> {
    fn new(module: &Module<'a>, types: &Types<'a>) -> Self {
        let calloc_f = Self::declare_function(
            &types.char_ptr_t,
            &[types.size_t_t.into(), types.size_t_t.into()],
            "calloc",
            module,
        );
        let free_f = Self::declare_void_function(&[types.char_ptr_t.into()], "free", module, types);
        let fputs_f = Self::declare_function(
            &types.int_t,
            &[types.char_ptr_t.into(), types.file_ptr_t.into()],
            "fputs",
            module,
        );
        let putchar_f =
            Self::declare_function(&types.int_t, &[types.int_t.into()], "putchar", module);
        let fflush_f =
            Self::declare_function(&types.int_t, &[types.file_ptr_t.into()], "fflush", module);

        let move_right_f = Self::declare_void_function(
            &[
                types.char_ptr_ptr_t.into(),
                types.size_t_ptr_t.into(),
                types.size_t_ptr_t.into(),
                types.size_t_t.into(),
            ],
            "moveRight",
            module,
            types,
        );
        let input_f = Self::declare_void_function(&[
                types.char_ptr_t.into(),
                types.size_t_t.into(),
                types.char_ptr_ptr_t.into(),
            ], "input", module, types);
        let move_right_until_zero_f = Self::declare_void_function(
            &[
                types.char_ptr_ptr_t.into(),
                types.size_t_ptr_t.into(),
                types.size_t_ptr_t.into(),
                types.size_t_t.into(),
            ],
            "moveRightUntilZero",
            module,
            types,
        );
        let move_left_until_zero_f = Self::declare_function(
            &types.bool_t,
            &[
                types.char_ptr_t.into(),
                types.size_t_ptr_t.into(),
                types.size_t_t.into(),
            ],
            "moveLeftUntilZero",
            module,
        );

        let move_value_right_f = Self::declare_void_function(
            &[
                types.char_ptr_ptr_t.into(),
                types.size_t_ptr_t.into(),
                types.size_t_t.into(),
                types.size_t_t.into(),
            ],
            "moveValueRight",
            module,
            types,
        );

        let move_value_left_f = Self::declare_function(
            &types.bool_t,
            &[
                types.char_ptr_t.into(),
                types.size_t_t.into(),
                types.size_t_t.into(),
            ],
            "moveValueLeft",
            module,
        );

        let main_f = Self::declare_function(&types.int_t, &[], "main", module);

        Self {
            calloc_f,
            free_f,
            fputs_f,
            putchar_f,
            fflush_f,
            move_right_f,
            input_f,
            move_right_until_zero_f,
            move_left_until_zero_f,
            move_value_right_f,
            move_value_left_f,
            main_f,
        }
    }

    fn declare_function<Type>(
        return_type: &Type,
        param_types: &[BasicMetadataTypeEnum<'a>],
        name: &str,
        module: &Module<'a>,
    ) -> FunctionValue<'a>
    where
        Type: BasicType<'a>,
    {
        let function_type = return_type.fn_type(param_types, false);
        module.add_function(name, function_type, None)
    }

    fn declare_void_function(
        param_types: &[BasicMetadataTypeEnum<'a>],
        name: &str,
        module: &Module<'a>,
        types: &Types<'a>,
    ) -> FunctionValue<'a> {
        let function_type = types.void_t.fn_type(param_types, false);
        module.add_function(name, function_type, None)
    }
}

pub struct CodeGen<'a> {
    instructions: Vec<Instruction>,
    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
    types: Types<'a>,
    globals: Globals<'a>,
    functions: Functions<'a>,
    main_error_block: BasicBlock<'a>,
    cells_alloca: PointerValue<'a>,
    cells_length_alloca: PointerValue<'a>,
    current_cell_alloca: PointerValue<'a>,
    input_buffer_alloca: PointerValue<'a>,
    multiplier_alloca: PointerValue<'a>,
}

impl<'a> CodeGen<'a> {
    pub fn new(instructions: Vec<Instruction>, input_file: &Path, context: &'a Context) -> Self {
        let module = context.create_module(input_file.file_stem().and_then(OsStr::to_str).unwrap());
        module.set_source_file_name(input_file.file_name().and_then(OsStr::to_str).unwrap());
        let builder = context.create_builder();

        let types = Types::new(context);
        let globals = Globals::new(context, &module, &types);
        let functions = Functions::new(&module, &types);

        let main_entry_block = context.append_basic_block(functions.main_f, "entry");
        let main_error_block = context.append_basic_block(functions.main_f, "error");

        builder.position_at_end(main_entry_block);

        let cells_alloca = builder.build_alloca(types.char_ptr_t, "cells");
        let cells_length_alloca = builder.build_alloca(types.size_t_t, "cellsLength");
        let current_cell_alloca = builder.build_alloca(types.size_t_t, "currentCell");
        let input_buffer_alloca = builder.build_alloca(types.char_ptr_t, "inputBuffer");
        let multiplier_alloca = builder.build_alloca(types.char_t, "multiplier");

        Self {
            instructions,
            context,
            module,
            builder,
            types,
            globals,
            functions,
            main_error_block,
            cells_alloca,
            cells_length_alloca,
            current_cell_alloca,
            input_buffer_alloca,
            multiplier_alloca,
        }
    }

    pub fn generate_module(&self) -> &Module<'a> {
        let args = &[
            self.types.size_t_t.const_int(256, false).into(),
            self.types.size_t_t.const_int(1, false).into(),
        ];
        let cells = self
            .builder
            .build_call(self.functions.calloc_f, args, "initialCells")
            .try_as_basic_value()
            .left()
            .unwrap();

        self.builder.build_store(self.cells_alloca, cells);
        self.builder.build_store(
            self.cells_length_alloca,
            self.types.size_t_t.const_int(256, false),
        );
        self.builder
            .build_store(self.current_cell_alloca, self.types.size_t_t.const_zero());
        self.builder
            .build_store(self.input_buffer_alloca, self.types.char_ptr_t.const_null());

        self.generate_instructions(&self.instructions);

        let return_block = self
            .context
            .append_basic_block(self.functions.main_f, "return");
        self.builder.build_unconditional_branch(return_block);

        let last_block = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(self.main_error_block);

        let casted_error_string = self.builder.build_bitcast(
            self.globals.error_string_v,
            self.types.char_ptr_t,
            "errorString",
        );
        let stderr_v = self
            .builder
            .build_load(self.globals.stderr_ptr_v.as_pointer_value(), "load");
        self.builder.build_call(
            self.functions.fputs_f,
            &[casted_error_string.into(), stderr_v.into()],
            "",
        );

        self.builder.build_unconditional_branch(return_block);

        self.builder.position_at_end(return_block);

        let phi = self.builder.build_phi(self.types.int_t, "returnValue");
        phi.add_incoming(&[
            (&self.types.int_t.const_int(0, false), last_block),
            (&self.types.int_t.const_int(1, false), self.main_error_block),
        ]);

        let cells = self.builder.build_load(self.cells_alloca, "load");
        self.builder
            .build_call(self.functions.free_f, &[cells.into()], "");

        let input_buffer = self.builder.build_load(self.input_buffer_alloca, "load");
        self.builder
            .build_call(self.functions.free_f, &[input_buffer.into()], "");

        self.builder.build_return(Some(&phi.as_basic_value()));

        if !self.functions.main_f.verify(true) {
            panic!("Could not verify main function")
        }

        self.module.verify().unwrap();

        &self.module
    }

    fn generate_instructions(&self, instructions: &[Instruction]) {
        let mut has_multiplier = false;
        for instruction in instructions.iter() {
            self.generate_instruction(instruction, &mut has_multiplier);
        }
    }

    fn generate_instruction(&self, instruction: &Instruction, has_multiplier: &mut bool) {
        match instruction {
            Instruction::MoveRight { amount } => {
                self.builder.build_call(
                    self.functions.move_right_f,
                    &[
                        self.cells_alloca.into(),
                        self.cells_length_alloca.into(),
                        self.current_cell_alloca.into(),
                        self.types.size_t_t.const_int(*amount as u64, false).into(),
                    ],
                    "",
                );
            }
            Instruction::MoveLeft { amount } => {
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell = self.builder.build_int_sub(
                    current_cell,
                    self.types.size_t_t.const_int(*amount as u64, false),
                    "decrementedCurrentCell",
                );

                let return_with_error = self.builder.build_int_compare(
                    IntPredicate::SLT,
                    current_cell,
                    self.types.size_t_t.const_zero(),
                    "returnWithError",
                );

                let move_left_block = self
                    .context
                    .prepend_basic_block(self.main_error_block, "moveLeft");

                self.builder.build_conditional_branch(
                    return_with_error,
                    self.main_error_block,
                    move_left_block,
                );
                self.builder.position_at_end(move_left_block);

                self.builder
                    .build_store(self.current_cell_alloca, current_cell);
            }
            Instruction::Increment { amount } | Instruction::Decrement { amount } => {
                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell_ptr = unsafe {
                    self.builder
                        .build_gep(cells, &[current_cell], "currentCellPtr")
                };

                let current_cell_value = self
                    .builder
                    .build_load(current_cell_ptr, "load")
                    .into_int_value();

                let mut amount = self.types.char_t.const_int(*amount as u64, false);

                if *has_multiplier {
                    let multiplier = self
                        .builder
                        .build_load(self.multiplier_alloca, "load")
                        .into_int_value();

                    amount = self.builder.build_int_mul(
                        amount,
                        multiplier,
                        "multipliedAmount",
                    );
                }

                let current_cell_value = if let Instruction::Increment { amount: _ } = instruction {
                    self.builder.build_int_add(
                        current_cell_value,
                        amount,
                        "incrementedCurrentCell",
                    )
                } else {
                    self.builder.build_int_sub(
                        current_cell_value,
                        amount,
                        "decrementedCurrentCell",
                    )
                };

                self.builder
                    .build_store(current_cell_ptr, current_cell_value);
            }
            Instruction::Output => {
                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell_ptr = unsafe {
                    self.builder
                        .build_gep(cells, &[current_cell], "currentCellPtr")
                };

                let current_cell_value = self
                    .builder
                    .build_load(current_cell_ptr, "load")
                    .into_int_value();

                let current_cell_value = self.builder.build_int_z_extend(
                    current_cell_value,
                    self.types.int_t,
                    "extendedCurrentCellValue",
                );

                self.builder
                    .build_call(self.functions.putchar_f, &[current_cell_value.into()], "");

                let stdout = self
                    .builder
                    .build_load(self.globals.stdout_ptr_v.as_pointer_value(), "load");

                self.builder
                    .build_call(self.functions.fflush_f, &[stdout.into()], "");
            }
            Instruction::Input => {
                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let args = &[
                    cells.into(),
                    current_cell.into(),
                    self.input_buffer_alloca.into(),
                ];
                self.builder.build_call(self.functions.input_f, args, "");
            }
            Instruction::Loop { instructions } => {
                let loop_block = self
                    .context
                    .prepend_basic_block(self.main_error_block, "loop");
                let then_block = self
                    .context
                    .prepend_basic_block(self.main_error_block, "then");
                let merge_block = self
                    .context
                    .prepend_basic_block(self.main_error_block, "merge");

                self.builder.build_unconditional_branch(loop_block);
                self.builder.position_at_end(loop_block);

                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell_ptr = unsafe {
                    self.builder
                        .build_gep(cells, &[current_cell], "currentCellPtr")
                };

                let current_cell_value = self
                    .builder
                    .build_load(current_cell_ptr, "load")
                    .into_int_value();

                let continue_loop = self.builder.build_int_compare(
                    IntPredicate::NE,
                    current_cell_value,
                    self.types.char_t.const_int(0, false),
                    "breakLoop",
                );

                self.builder
                    .build_conditional_branch(continue_loop, then_block, merge_block);

                self.builder.position_at_end(then_block);

                self.generate_instructions(instructions);

                self.builder.build_unconditional_branch(loop_block);
                self.builder.position_at_end(merge_block);
            }
            Instruction::MoveRightUntilZero { step_size } => {
                self.builder.build_call(
                    self.functions.move_right_until_zero_f,
                    &[
                        self.cells_alloca.into(),
                        self.cells_length_alloca.into(),
                        self.current_cell_alloca.into(),
                        self.types.size_t_t.const_int(*step_size as u64, false).into(),
                    ],
                    "",
                );
            }
            Instruction::MoveLeftUntilZero { step_size } => {
                let cells = self.builder.build_load(self.cells_alloca, "load");

                let return_with_error = self
                    .builder
                    .build_call(
                        self.functions.move_left_until_zero_f,
                        &[
                            cells.into(),
                            self.current_cell_alloca.into(),
                            self.types.size_t_t.const_int(*step_size as u64, false).into(),
                        ],
                        "returnWithError",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
                    .into_int_value();

                let continue_block = self
                    .context
                    .prepend_basic_block(self.main_error_block, "continue");

                self.builder.build_conditional_branch(
                    return_with_error,
                    self.main_error_block,
                    continue_block,
                );
                self.builder.position_at_end(continue_block);
            }
            Instruction::SetToZero => {
                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell_ptr = unsafe {
                    self.builder
                        .build_gep(cells, &[current_cell], "currentCellPtr")
                };

                self.builder
                    .build_store(current_cell_ptr, self.types.char_t.const_zero());
            }
            Instruction::SetMultiplier => {
                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();

                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell_ptr = unsafe {
                    self.builder
                        .build_gep(cells, &[current_cell], "currentCellPtr")
                };

                let multiplier = self
                    .builder
                    .build_load(current_cell_ptr, "multiplier")
                    .into_int_value();

                self.builder.build_store(self.multiplier_alloca, multiplier);

                *has_multiplier = true;
            }
            Instruction::ResetMultiplierAndSetToZero => {
                let cells = self
                    .builder
                    .build_load(self.cells_alloca, "load")
                    .into_pointer_value();

                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load")
                    .into_int_value();

                let current_cell_ptr = unsafe {
                    self.builder
                        .build_gep(cells, &[current_cell], "currentCellPtr")
                };

                self.builder
                    .build_store(current_cell_ptr, self.types.char_t.const_zero());

                *has_multiplier = false;
            }
            Instruction::MoveValueRight { amount } => {
                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load");

                self.builder.build_call(
                    self.functions.move_value_right_f,
                    &[
                        self.cells_alloca.into(),
                        self.cells_length_alloca.into(),
                        current_cell.into(),
                        self.types.size_t_t.const_int(*amount as u64, false).into(),
                    ],
                    "",
                );
            }
            Instruction::MoveValueLeft { amount } => {
                let cells = self.builder.build_load(self.cells_alloca, "load");

                let current_cell = self
                    .builder
                    .build_load(self.current_cell_alloca, "load");

                let return_with_error = self.builder.build_call(
                    self.functions.move_value_left_f,
                    &[
                        cells.into(),
                        current_cell.into(),
                        self.types.size_t_t.const_int(*amount as u64, false).into(),
                    ],
                    "",
                )
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value();

                let continue_block = self
                    .context
                    .prepend_basic_block(self.main_error_block, "continue");

                self.builder.build_conditional_branch(
                    return_with_error,
                    self.main_error_block,
                    continue_block,
                );
                self.builder.position_at_end(continue_block);
            }
        }
    }
}
