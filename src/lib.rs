mod cli {
    use std::io::stdin;

    pub struct  CLI {
        arguments: Vec<String>,
    }
    

    impl CLI {
        pub fn contains_flag(&self, flag: &str) -> bool {
            self.arguments.iter().any(|arg| arg.to_string()=="--".to_string() + flag)
        }

        pub fn get_command(&self) -> &str {
            &self.arguments[1]
        }

        pub fn get_param(&self, param_name: &str) -> String {
            for arg_index in 0..self.arguments.len() {
                if self.arguments[arg_index] == "-".to_string() + param_name {
                    return (self.arguments[arg_index + 1]).to_string();
                }
            }
            return "".to_string()
        }

        pub fn ask(&self, question: &str) -> String {
            let mut awnser = String::new();
            println!("{}", question);
            stdin().read_line(&mut awnser).expect("Failed to read line. Try Again.");
            awnser
        }

        pub fn read_required(&self, flag: &str, description: &str) -> String {
            let mut val: String = self.get_param(flag);
            if val.is_empty() {
                val = self.ask(description);
                if val.is_empty() {
                    println!("Please try again by actually entering a value.");
                    std::process::exit(2);
                }
            }
            val
        }
    }
}