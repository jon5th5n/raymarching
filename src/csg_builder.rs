use std::{collections::HashMap, slice::Iter};

pub fn run() -> Scene {
    println!("csg builder...");

    let prims = HashMap::from([
        (
            "sphere".to_string(),
            Primitive {
                primitive_func: "sdf_sphere([point], {vec3}, {float})".to_string(),
            },
        ),
        (
            "box".to_string(),
            Primitive {
                primitive_func: "sdf_box([point], {vec3}, {vec3})".to_string(),
            },
        ),
        (
            "torus".to_string(),
            Primitive {
                primitive_func: "sdf_torus([point], {vec3}, {vec2})".to_string(),
            },
        ),
    ]);

    let alters = HashMap::from([
        (
            "scale".to_string(),
            Alterator {
                input_func: Some("op_i_scale([point], {float})".to_string()),
                output_func: Some("op_o_scale([sd], {float})".to_string()),
            },
        ),
        (
            "trans".to_string(),
            Alterator {
                input_func: Some("op_i_trans([point], {vec3})".to_string()),
                output_func: None,
            },
        ),
        (
            "rotang".to_string(),
            Alterator {
                input_func: Some("op_i_rot_ang([point], {vec3})".to_string()),
                output_func: None,
            },
        ),
        (
            "transrotang".to_string(),
            Alterator {
                input_func: Some("op_i_transrot_ang([point], {vec3}, {vec3})".to_string()),
                output_func: None,
            },
        ),
    ]);

    let combs = HashMap::from([
        (
            "UNION".to_string(),
            Combinator {
                combination_func: "com_union([], [])".to_string(),
            },
        ),
        (
            "INTER".to_string(),
            Combinator {
                combination_func: "com_intersect([], [])".to_string(),
            },
        ),
        (
            "SUBST".to_string(),
            Combinator {
                combination_func: "com_substract([], [])".to_string(),
            },
        ),
    ]);

    let mut scene = Scene {
        building_blocks_path: "./src/sdf.fs".to_string(),
        primitives: prims,
        combinators: combs,
        alterators: alters,
        scene: vec![],
        scene_sdf_uneval: "".to_string(),
        variables: HashMap::new(),
    };
    scene.scene_from_instructions(std::fs::read_to_string("./instructions_dyncomp.txt").unwrap());

    scene.generate_scene_sdf();

    // *scene.get_variable_float_mut("scale1").unwrap() = 1.0;
    // *scene.get_variable_float_mut("scale2").unwrap() = 1.0;

    // *scene.get_variable_float_mut("radius1").unwrap() = 15.0;
    // *scene.get_variable_float_mut("radius2").unwrap() = 15.0;
    // *scene.get_variable_float_mut("radius3").unwrap() = 15.0;

    // println!("{:?}", scene.variables);
    // println!("{}", scene.get_scene_sdf_eval());
    // println!("{}", scene.get_scene_sdf_eval());
    scene
}

//=====

pub struct Scene {
    building_blocks_path: String,

    primitives: HashMap<String, Primitive>,
    combinators: HashMap<String, Combinator>,
    alterators: HashMap<String, Alterator>,

    scene: Vec<CSG>,

    scene_sdf_uneval: String,
    variables: HashMap<String, f32>,
}
impl Scene {
    pub fn new(
        building_blocks_path: &str,
        primitives: HashMap<String, Primitive>,
        combinators: HashMap<String, Combinator>,
        alterators: HashMap<String, Alterator>,
        scene: Vec<CSG>,
    ) -> Self {
        Self {
            building_blocks_path: building_blocks_path.to_string(),
            primitives,
            combinators,
            alterators,
            scene,
            scene_sdf_uneval: "".to_string(),
            variables: HashMap::new(),
        }
    }

    pub fn scene_from_instructions(&mut self, instructions: String) {
        let mut instructions = instructions;
        self.scene.clear();
        while let Some(opening) = instructions.find('{') {
            instructions.replace_range(opening..=opening, " ");

            let closing = instructions
                .find('}')
                .expect("missing closing bracket in instructions!");
            instructions.replace_range(closing..=closing, " ");

            let csg_instructions = instructions[(opening + 1)..closing].to_string();
            let csg_instructions = csg_instructions
                .split('\n')
                .filter(|i| !i.is_empty())
                .collect::<Vec<_>>();

            let mut csg_tree: Vec<CSGNode> = Vec::new();
            for csg_instruction in csg_instructions {
                let csg_instruction = csg_instruction
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join("");

                let csg_node = self.csg_node_from_instruction(csg_instruction);

                csg_tree.push(csg_node);
            }

            self.scene.push(CSG::new(csg_tree));
        }

        // csgs
    }

    fn csg_node_from_instruction(&self, instruction: String) -> CSGNode {
        let mut instruction = instruction.split('+');
        let (node, alters) = (
            instruction
                .next()
                .expect("maybe missing alterators!")
                .to_string(),
            instruction
                .next()
                .expect("maybe missing alterators!")
                .to_string(),
        );

        let alters = alters
            .strip_prefix('[')
            .expect("use '[]' to encase alterators!")
            .strip_suffix(']')
            .expect("use '[]' to encase alterators!")
            .split(';')
            .filter(|i| !i.is_empty())
            .map(|alter| {
                let (alter_name, alter_params) =
                    Self::prim_instruction_name_and_params(alter.to_string());
                self.alterators
                    .get(&alter_name)
                    .expect("alterator missing in scene struct!")
                    .eval(alter_params)
            })
            .collect::<Vec<_>>();

        match node.chars().next().unwrap().is_uppercase() {
            true => CSGNode::Combinator(CombinatorNode::new(
                self.combinators
                    .get(&node)
                    .expect("combinator missing in scene struct!")
                    .clone(),
                alters,
            )),
            false => {
                let (prim_name, prim_params) = Self::prim_instruction_name_and_params(node);
                let prim = self
                    .primitives
                    .get(&prim_name)
                    .expect("primitive missing in scene struct!")
                    .eval(prim_params);
                CSGNode::Primitive(PrimitiveNode::new(prim, alters))
            }
        }
    }

    fn prim_instruction_name_and_params(instruction: String) -> (String, Vec<Param>) {
        let mut instruction = instruction.split('(');
        let (name, params) = (
            instruction
                .next()
                .expect("syntax error! should look like 'foo()'")
                .to_string(),
            instruction
                .next()
                .expect("syntax error! should look like 'foo()'")
                .to_string(),
        );

        let params = params
            .strip_suffix(')')
            .expect("syntax error! should look like 'foo()'")
            .split(',')
            .map(|param| Self::parse_param(param.to_string()))
            .collect::<Vec<_>>();

        (name, params)
    }

    fn parse_param(param: String) -> Param {
        if let Ok(float) = param.parse::<f32>() {
            return Param::Float(float);
        }

        if let Some(param) = param.strip_prefix('<') {
            let param = param
                .strip_suffix('>')
                .expect("missing closing '>'!")
                .split('|')
                .collect::<Vec<_>>();

            return match param.len() {
                2 => Param::Vec2([
                    param[0].parse().expect("bad vector!"),
                    param[1].parse().expect("bad vector!"),
                ]),
                3 => Param::Vec3([
                    param[0].parse().expect("bad vector!"),
                    param[1].parse().expect("bad vector!"),
                    param[2].parse().expect("bad vector!"),
                ]),
                4 => Param::Vec4([
                    param[0].parse().expect("bad vector!"),
                    param[1].parse().expect("bad vector!"),
                    param[2].parse().expect("bad vector!"),
                    param[3].parse().expect("bad vector!"),
                ]),
                _ => panic!("invalid vector length! valid vectors have size 2-4."),
            };
        }

        Param::Variable(param)
    }

    pub fn generate_scene_sdf(&mut self) {
        let result = match self.scene.len() {
            0 => "".to_string(),
            1 => self.scene[0].get_primitive_eval().primitive_func,
            2.. => {
                let mut res = format!(
                    "({one}.w < {two}.w ? {one} : {two})",
                    one = self.scene[0].get_primitive_eval().primitive_func,
                    two = self.scene[1].get_primitive_eval().primitive_func
                );

                for i in 2..self.scene.len() {
                    res = format!(
                        "({one}.w < {two}.w ? {one} : {two})",
                        one = self.scene[i].get_primitive_eval().primitive_func,
                        two = res
                    );
                }

                res
            }
            _ => panic!("dafuq!"),
        };

        self.scene_sdf_uneval = format!("vec4 sdf(vec3 point){{\n\treturn {};\n}}", result);

        self.create_sdf_variables();
    }

    fn create_sdf_variables(&mut self) {
        let mut sdf = self.scene_sdf_uneval.clone();

        while let Some(opening) = sdf.find('$') {
            sdf.replace_range(opening..=opening, " ");

            let closing = sdf.find('$').expect("missing closing '$' for variable!");
            sdf.replace_range(closing..=closing, " ");

            let var_name = sdf[(opening + 1)..closing].to_string();
            if self.variables.contains_key(&var_name) {
                continue;
            }
            self.variables.insert(var_name, 0.0);
        }
    }

    pub fn get_scene_sdf_eval(&self) -> String {
        let mut result = self.scene_sdf_uneval.clone();

        while let Some(opening) = result.find('$') {
            result.replace_range(opening..=opening, " ");

            let closing = result.find('$').expect("missing closing '$' for variable!");

            let var_name = &result[(opening + 1)..closing];
            let var_val = self
                .variables
                .get(var_name)
                .expect("missing matching variable for evaluating of sdf!");

            result.replace_range(opening..=closing, &format!("{:?}", var_val));
        }
        result = result.replace("[point]", "point");

        result = format!(
            "{}\n\n{}",
            std::fs::read_to_string(&self.building_blocks_path).unwrap(),
            result
        );

        result
    }

    pub fn get_scene_sdf_eval_test(&self) -> String {
        let mut result = self.scene_sdf_uneval.clone();

        let mut variable_header = "".to_string();

        while let Some(opening) = result.find('$') {
            result.replace_range(opening..=opening, " ");

            let closing = result.find('$').expect("missing closing '$' for variable!");

            let var_name = &result[(opening + 1)..closing];
            let var_val = self
                .variables
                .get(var_name)
                .expect("missing matching variable for evaluating of sdf!");

            if !variable_header.contains(var_name) {
                variable_header = format!("{}uniform float {};\n", variable_header, var_name);
            }

            result.replace_range(opening..=closing, &var_name.to_string());
        }
        result = result.replace("[point]", "point");

        result = format!(
            "{}\n\n{}\n\n{}",
            std::fs::read_to_string(&self.building_blocks_path).unwrap(),
            variable_header,
            result
        );

        result
    }

    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    pub fn get_variables(&self) -> &HashMap<String, f32> {
        &self.variables
    }

    pub fn get_variable_float(&self, variable: &str) -> Option<&f32> {
        self.variables.get(variable)
    }
    pub fn get_variable_float_mut(&mut self, variable: &str) -> Option<&mut f32> {
        self.variables.get_mut(variable)
    }

    // fn get_variable_vec3(&self, variable: &str) -> Option<[&f32; 3]> {
    //     let vec3 = [
    //         self.variables.get(&format!("{}_x", variable)),
    //         self.variables.get(&format!("{}_y", variable)),
    //         self.variables.get(&format!("{}_z", variable)),
    //     ];

    //     if vec3.contains(&None) {
    //         return None;
    //     }

    //     Some([vec3[0].unwrap(), vec3[1].unwrap(), vec3[2].unwrap()])
    // }
}

//=====

pub enum CSGNode {
    Combinator(CombinatorNode),
    Primitive(PrimitiveNode),
}

pub struct CombinatorNode {
    combinator: Combinator,
    alterations: Vec<AlteratorEval>,
}
impl CombinatorNode {
    fn new(combinator: Combinator, alterations: Vec<AlteratorEval>) -> Self {
        Self {
            combinator,
            alterations,
        }
    }
}

pub struct PrimitiveNode {
    primitive: PrimitiveEval,
    alterations: Vec<AlteratorEval>,
}
impl PrimitiveNode {
    fn new(primitive: PrimitiveEval, alterations: Vec<AlteratorEval>) -> Self {
        Self {
            primitive,
            alterations,
        }
    }
}

pub struct CSG {
    tree: Vec<CSGNode>,
}
impl CSG {
    fn new(tree: Vec<CSGNode>) -> Self {
        Self { tree }
    }

    fn get_primitive_eval(&self) -> PrimitiveEval {
        Self::step_tree(&mut self.tree.iter())
    }

    fn step_tree(mut tree: &mut Iter<CSGNode>) -> PrimitiveEval {
        match tree.next() {
            Some(CSGNode::Primitive(prim)) => prim.primitive.apply_alterations(&prim.alterations),
            Some(CSGNode::Combinator(comb)) => comb
                .combinator
                .combine(Self::step_tree(&mut tree), Self::step_tree(&mut tree))
                .apply_alterations(&comb.alterations),
            None => panic!("unexpected end while parsing CSG-Tree!"),
        }
    }
}

//=====

#[derive(Clone)]
pub enum Param {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Variable(String),
}

fn eval_function(func: String, mut params: Vec<Param>) -> String {
    params.reverse();
    let mut result = func;

    while let Some(opening) = result.find("{") {
        let param = params.pop().expect("too few arguments supplied!");

        let mut pre_variable_size = 5;
        let param_str = match (&result[(opening + 1)..=(opening + 5)], param) {
            ("float", Param::Float(float)) => format!("{:?}", float),
            ("float", Param::Variable(variable)) => format!("${}$", variable),
            ("vec2}", Param::Vec2(vec2)) => {
                pre_variable_size = 4;
                format!("vec2({:?},{:?})", vec2[0], vec2[1])
            }
            ("vec2}", Param::Variable(variable)) => {
                pre_variable_size = 4;
                format!("vec3(${}_x$,${}_y$)", variable, variable)
            }
            ("vec3}", Param::Vec3(vec3)) => {
                pre_variable_size = 4;
                format!("vec3({:?},{:?},{:?})", vec3[0], vec3[1], vec3[2])
            }
            ("vec3}", Param::Variable(variable)) => {
                pre_variable_size = 4;
                format!("vec3(${}_x$,${}_y$,${}_z$)", variable, variable, variable)
            }
            ("vec4}", Param::Vec4(vec4)) => {
                pre_variable_size = 4;
                format!(
                    "vec3({:?},{:?},{:?},{:?})",
                    vec4[0], vec4[1], vec4[2], vec4[3]
                )
            }
            ("vec4}", Param::Variable(variable)) => {
                pre_variable_size = 4;
                format!(
                    "vec3(${}_x$,${}_y$,${}_z$,${}_w$)",
                    variable, variable, variable, variable
                )
            }
            _ => panic!("invalid variable type or mismatched type order!"),
        };

        result.replace_range((opening)..=(opening + pre_variable_size + 1), &param_str);
    }

    result
}

pub struct Primitive {
    primitive_func: String,
}
impl Primitive {
    pub fn eval(&self, params: Vec<Param>) -> PrimitiveEval {
        PrimitiveEval {
            primitive_func: eval_function(self.primitive_func.clone(), params.clone()),
        }
    }
}

pub struct PrimitiveEval {
    primitive_func: String,
}
impl PrimitiveEval {
    pub fn apply_alterations(&self, alterations: &[AlteratorEval]) -> PrimitiveEval {
        let mut result = self.primitive_func.clone();

        for alteration in alterations {
            if let Some(input_func) = &alteration.input_func {
                result = result.replace("[point]", input_func);
            }
            if let Some(output_func) = &alteration.output_func {
                result = output_func.replace("[sd]", &result);
            }
        }

        PrimitiveEval {
            primitive_func: result,
        }
    }
}

pub struct Alterator {
    input_func: Option<String>,
    output_func: Option<String>,
}
impl Alterator {
    pub fn eval(&self, params: Vec<Param>) -> AlteratorEval {
        AlteratorEval {
            input_func: match self.input_func.clone() {
                Some(func) => Some(eval_function(func, params.clone())),
                None => None,
            },
            output_func: match self.output_func.clone() {
                Some(func) => Some(eval_function(func, params.clone())),
                None => None,
            },
        }
    }
}

pub struct AlteratorEval {
    input_func: Option<String>,
    output_func: Option<String>,
}

#[derive(Clone)]
pub struct Combinator {
    combination_func: String,
}
impl Combinator {
    fn combine(&self, one: PrimitiveEval, two: PrimitiveEval) -> PrimitiveEval {
        let result = self.combination_func.replacen("[]", &one.primitive_func, 1);
        let result = result.replacen("[]", &two.primitive_func, 1);

        PrimitiveEval {
            primitive_func: result,
        }
    }
}

//=====

pub struct SimpleScene {
    pub building_blocks_path: String,
    pub variables: HashMap<String, f32>,
    pub scene_sdf: String,
}
impl SimpleScene {
    pub fn generate_glsl(&self) -> String {
        let mut result = self.scene_sdf.clone();

        while let Some(start) = result.find('$') {
            result = result.replacen('$', " ", 1);
            let end = result.find('$').expect("missing closing '$' for variable!");

            let variable = result[(start + 1)..end].to_string();
            let value = self
                .variables
                .get(&variable)
                .expect("'scene_sdf' includes variable not contained in variable hashmap!");

            result.replace_range(start..=end, &format!("{:?}", value));
        }

        result = format!(
            "{}\n{}",
            std::fs::read_to_string(&self.building_blocks_path).unwrap(),
            result
        );

        result
    }
}

//=====
