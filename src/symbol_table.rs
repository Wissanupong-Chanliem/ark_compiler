use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};
use crate::tokenizer::DataType;
// #[derive(Debug,Clone)]
// pub enum Scope {
//     Function(String),
//     Block(usize),
//     Global,
// }
// #[derive(Debug,Clone)]
// pub struct VarAttribute {
//     pub data_type:Option<DataType>,
//     pub size:Option<u32>,
//     pub dimension:Option<u32>,
//     pub line_declare:Option<u32>,
//     pub line_ref:Vec<u32>,
// }

// impl VarAttribute {
//     pub fn set_data_type(&mut self,dt:DataType) -> &mut Self{
//         self.data_type = Some(dt);
//         self
//     }
//     pub fn set_size(&mut self,size_in_byte:u32) -> &mut Self{
//         self.size = Some(size_in_byte);
//         self
//     }
//     pub fn set_dimension(&mut self,dimension:u32) -> &mut Self{
//         self.dimension = Some(dimension);
//         self
//     }
//     pub fn set_line_declare(&mut self,line_declare:u32) -> &mut Self{
//         self.line_declare = Some(line_declare);
//         self
//     }
//     pub fn push_line_ref(&mut self,line_ref:u32) -> &mut Self{
//         self.line_ref.push(line_ref);
//         self
//     }
// }

// #[derive(Debug,Clone)]

// pub struct FuncAttribute {
//     line_declare:Option<u32>,
//     line_used:Vec<u32>,
//     parameter:Vec<(DataType,String)>,
//     return_type:DataType,
//     func_table:RefCell<Rc<SymbolTable>>
// }

// impl FuncAttribute {
//     pub fn set_return_type(&mut self,dt:DataType) -> &mut Self{
//         self.return_type = dt;
//         self
//     }
//     pub fn set_line_declare(&mut self,line_declare:u32) -> &mut Self{
//         self.line_declare = Some(line_declare);
//         self
//     }
//     pub fn push_line_used(&mut self,line_used:u32) -> &mut Self{
//         self.line_used.push(line_used);
//         self
//     }
//     pub fn push_parameter(&mut self,param:(DataType,String)) -> &mut Self{
//         self.parameter.push(param);
//         self
//     }
//     pub fn get_mut_table(&mut self) -> &mut SymbolTable{
//         &mut self.func_table
//     }
// }

// #[derive(Debug,Clone)]
// pub struct SymbolTable{
//     scope:Scope,
//     var_table:HashMap<String,VarAttribute>,
//     func_table:HashMap<String,FuncAttribute>,
//     inner_scope:RefCell<Vec<Rc<SymbolTable>>>,
//     higher_scope:RefCell<Weak<SymbolTable>>
// }

// impl SymbolTable {
//     pub fn new(scope:Scope) -> SymbolTable{
//         SymbolTable{
//             scope,
//             var_table:HashMap::new(),
//             func_table:HashMap::new(),
//             inner_scope:RefCell::new(vec![]),
//             higher_scope:RefCell::new(Weak::new())
//         }
//     }

//     fn type_to_size(dt:DataType) -> u32{
//         match dt {
//             DataType::Void => 0,
//             DataType::I8 => 1,
//             DataType::I16 => 2,
//             DataType::I32 => 4,
//             DataType::I64 => 8,
//             DataType::U8 => 1,
//             DataType::U16 => 2,
//             DataType::U32 => 4,
//             DataType::U64 => 8,
//             DataType::F32 => 4,
//             DataType::F64 => 8,
//             DataType::String => 1,
//             DataType::Boolean => 1,
//         }
//     }

//     pub fn insert_var(&mut self,identifier:String) -> &mut VarAttribute {
//         self.var_table.insert(
//             identifier.clone(), 
//             VarAttribute {
//                 data_type: None,
//                 size: None,
//                 dimension: None,
//                 line_declare: None,
//                 line_ref: vec![] 
//             }
//         );
//         return self.var_table.get_mut(&identifier).unwrap();
//     }

//     pub fn look_up_var(&mut self,identifier:&String) -> Option<&mut VarAttribute>{
//         return self.var_table.get_mut(identifier);
//     }

//     pub fn insert_func(self:&Rc<Self>,identifier:String) -> &mut FuncAttribute{
        
//         let mut func_att = FuncAttribute {
//             line_declare: None,
//             line_used: vec![],
//             parameter: vec![],
//             return_type: DataType::Void,
//             func_table: RefCell::new(
//                 Rc::new(
//                     SymbolTable::new(Scope::Function(identifier.clone()))
//                 )
//             )
//         };
//         *(func_att.func_table.borrow_mut()).higher_scope = Rc::downgrade(self);
//         self.func_table.insert(
//             identifier.clone(), 
//             func_att
            
//         );
//         return self.func_table.get_mut(&identifier).unwrap();
//     }

//     pub fn look_up_func(&mut self,identifier:&String) -> Option<&mut FuncAttribute>{
//         return self.func_table.get_mut(identifier);
//     }

//     pub fn insert_block(&mut self) -> &mut SymbolTable{
//         let mut inner = SymbolTable::new(Scope::Block(self.inner_scope.len()));
//         inner.borrow_mut().higher_scope = Some(Rc::downgrade(&self));
//         self.inner_scope.push(
//             inner
//         );
//         return self.inner_scope.last_mut().unwrap();
//     }

//     pub fn look_up_block(&mut self,id:usize) -> Option<&mut SymbolTable>{
//         return self.inner_scope.get_mut(id);
//     }
//     // pub fn insert_var(&mut self,identifier:String) {
//     //     self.var_table.insert(
//     //         identifier, 
//     //         VarAttribute {
//     //             data_type: None,
//     //             size: None,
//     //             dimension: None,
//     //             line_declare: None,
//     //             line_ref: vec![] 
//     //         }
//     //     );
//     // }
// }


#[derive(Debug,Clone,PartialEq, Eq)]
pub enum Scope {
    Function(String),
    Block(usize),
    Global,
}
#[derive(Debug,Clone)]
pub struct VarAttribute {
    pub data_type:Option<DataType>,
    pub size:Option<u32>,
    pub dimension:Option<u32>,
    pub line_declare:Option<u32>,
    pub line_ref:Vec<u32>,
}

impl VarAttribute {
    pub fn set_data_type(&mut self,dt:DataType) -> &mut Self{
        self.data_type = Some(dt);
        self
    }
    pub fn set_size(&mut self,size_in_byte:u32) -> &mut Self{
        self.size = Some(size_in_byte);
        self
    }
    pub fn set_dimension(&mut self,dimension:u32) -> &mut Self{
        self.dimension = Some(dimension);
        self
    }
    pub fn set_line_declare(&mut self,line_declare:u32) -> &mut Self{
        self.line_declare = Some(line_declare);
        self
    }
    pub fn push_line_ref(&mut self,line_ref:u32) -> &mut Self{
        self.line_ref.push(line_ref);
        self
    }
}

#[derive(Debug,Clone)]
pub struct FuncAttribute {
    line_declare:Option<u32>,
    line_used:Vec<u32>,
    parameter:Vec<(DataType,String)>,
    return_type:DataType,
    func_table:RefCell<Rc<SymbolTable>>
}

impl FuncAttribute {
    pub fn set_return_type(&mut self,dt:DataType) -> &mut Self{
        self.return_type = dt;
        self
    }
    pub fn set_line_declare(&mut self,line_declare:u32) -> &mut Self{
        self.line_declare = Some(line_declare);
        self
    }
    pub fn push_line_used(&mut self,line_used:u32) -> &mut Self{
        self.line_used.push(line_used);
        self
    }
    pub fn push_parameter(&mut self,param:(DataType,String)) -> &mut Self{
        self.parameter.push(param);
        self
    }
}

#[derive(Debug)]
pub struct SymbolTable{
    scope:Scope,
    var_table:RefCell<HashMap<String,VarAttribute>>,
    func_table:RefCell<HashMap<String,FuncAttribute>>,
    inner_scope:RefCell<Vec<Rc<SymbolTable>>>,
    higher_scope:RefCell<Weak<SymbolTable>>
}

impl SymbolTable{
    pub fn new(scope:Scope) -> Self{
        SymbolTable {
            scope,
            var_table: RefCell::new(HashMap::new()),
            func_table:RefCell::new(HashMap::new()),
            inner_scope: RefCell::new(vec![]),
            higher_scope: RefCell::new(Weak::new())
        }
    }
    pub fn insert_var(
        self:& Rc<Self>,
        identifier:String,
    ){
        self.var_table.borrow_mut().insert(
            identifier.clone(), 
            VarAttribute {
                data_type: None,
                size: None,
                dimension: None,
                line_declare: None,
                line_ref: vec![] 
            }
        );
    }

    pub fn update_var(
        self:& Rc<Self>,
        identifier:String,
        data_type:Option<DataType>,
        size:Option<u32>,
        dimension:Option<u32>,
        line_declare:Option<u32>,
    ){
        let mut table = self.var_table.borrow_mut();
        let var_attribute = match table.get_mut(&identifier){
            Some(at) => at,
            None => panic!("Try to update unintialize entry in symbol table"),
        };
        if data_type.is_some() {
            var_attribute.data_type = data_type;
        }
        if size.is_some() {
            var_attribute.size = size;
        }
        if dimension.is_some() {
            var_attribute.dimension = dimension;
        }
        if line_declare.is_some() {
            var_attribute.line_declare = line_declare;
        }
    }
    pub fn update_var_at(
        self:& Rc<Self>,
        scope:Scope,
        identifier:String,
        data_type:Option<DataType>,
        size:Option<u32>,
        dimension:Option<u32>,
        line_declare:Option<u32>,
    ){

        let mut iter = Rc::clone(&self);
        loop {
            if iter.scope == scope{
                
                break;
            }
            if iter.scope == Scope::Global{
                
            }
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        iter.update_var(identifier, data_type, size, dimension, line_declare)
    }

    pub fn var_push_line_ref(self:& Rc<Self>,identifier:String,line_ref:u32){
        self.var_table.borrow_mut().get_mut(&identifier).unwrap().push_line_ref(line_ref);
    }

    pub fn var_push_line_ref_at(self:& Rc<Self>,scope:Scope,identifier:String,line_ref:u32){
        let mut iter = Rc::clone(&self);
        loop {
            if iter.scope == scope{
                
                break;
            }
            if iter.scope == Scope::Global{
                
            }
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        iter.var_push_line_ref(identifier, line_ref);
    }

    pub fn insert_func(
        self:& Rc<Self>,
        identifier:String,
        
    )  -> Rc<SymbolTable> {
        let func_attribute = FuncAttribute {
            return_type: DataType::Void,
            line_declare: None,
            parameter: vec![],
            line_used: vec![],
            func_table: RefCell::new(Rc::new(
                SymbolTable {
                    scope: Scope::Function(identifier.clone()),
                    var_table: RefCell::new(HashMap::new()),
                    func_table: RefCell::new(HashMap::new()),
                    inner_scope: RefCell::new(vec![]),
                    higher_scope: RefCell::new(Rc::downgrade(&self))
                }
            ))
        };
        let rc_func = Rc::clone(&func_attribute.func_table.borrow());
        self.inner_scope.borrow_mut().push(rc_func.clone());

        self.func_table.borrow_mut().insert(
            identifier.clone(), 
            func_attribute
        );
        return Rc::clone(&rc_func);
    }

    pub fn update_func(
        self:& Rc<Self>,
        identifier:String,
        return_type:Option<DataType>,
        line_declare:Option<u32>,
        
    ){
        let mut table = self.func_table.borrow_mut();
        let func_attribute = match table.get_mut(&identifier){
            Some(at) => at,
            None => panic!("Try to update unintialize entry in symbol table"),
        };
        if return_type.is_some() {
            func_attribute.return_type = return_type.unwrap();
        }
        if line_declare.is_some() {
            func_attribute.line_declare = line_declare;
        }
    }

    pub fn update_func_at(
        self:& Rc<Self>,
        scope:Scope,
        identifier:String,
        return_type:Option<DataType>,
        line_declare:Option<u32>,
        
    ){

        let mut iter = Rc::clone(&self);
        loop {
            if iter.scope == scope{
                
                break;
            }
            if iter.scope == Scope::Global{
                
            }
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        iter.update_func(identifier, return_type, line_declare);
    }

    pub fn func_push_line_ref(self:& Rc<Self>,identifier:String,line_ref:u32){
        self.var_table.borrow_mut().get_mut(&identifier).unwrap().push_line_ref(line_ref);
    }

    pub fn func_push_param(self:& Rc<Self>,identifier:String,data:(DataType,String)){
        self.func_table.borrow_mut().get_mut(&identifier).unwrap().push_parameter(data);
    }

    pub fn func_push_line_ref_at(self:& Rc<Self>,scope:Scope,identifier:String,line_ref:u32){
        let mut iter = Rc::clone(&self);
        loop {
            if iter.scope == scope{
                
                break;
            }
            if iter.scope == Scope::Global{
                
            }
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        iter.func_push_line_ref(identifier, line_ref)
    }

    pub fn func_push_param_at(self:& Rc<Self>,scope:Scope,identifier:String,data:(DataType,String)){
        let mut iter = Rc::clone(&self);
        loop {
            if iter.scope == scope{
                
                break;
            }
            if iter.scope == Scope::Global{
                
            }
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        iter.func_push_param(identifier, data);
    }

    pub fn lookup_var(self:&Rc<Self>,identifier:String) -> Option<(VarAttribute,Scope)> {
        let mut iter = Rc::clone(&self);
        loop {
            println!("{:#?}",iter);
            if iter.var_table.borrow().contains_key(&identifier) {
                return Some((iter.var_table.borrow().get(&identifier).unwrap().clone(),iter.scope.clone()));
            }
            if iter.scope == Scope::Global {
                
                break;
            }
            
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        return None;
    }

    pub fn lookup_func(self:&Rc<Self>,identifier:String) -> Option<(FuncAttribute,Scope)> {
        let mut iter = Rc::clone(&self);
        loop {
            println!("{:#?}",iter);
            if iter.func_table.borrow().contains_key(&identifier) {
                return Some((iter.func_table.borrow().get(&identifier).unwrap().clone(),iter.scope.clone()));
            }
            if iter.scope == Scope::Global {
                
                break;
            }
            
            let temp_iter = iter.higher_scope.borrow().upgrade().unwrap();
            iter = temp_iter;
        }
        return None;
    }

    pub fn insert_block_scope(self:&Rc<Self>) -> Rc<SymbolTable>{
        let child = Rc::new(
            SymbolTable {
                scope: Scope::Block(self.inner_scope.borrow().len()),
                var_table: RefCell::new(HashMap::new()),
                func_table: RefCell::new(HashMap::new()),
                inner_scope: RefCell::new(vec![]),
                higher_scope: RefCell::new(Rc::downgrade(&self))
            }
        );
        self.inner_scope.borrow_mut().push(Rc::clone(&child));
        return Rc::clone(&child);
    }
}