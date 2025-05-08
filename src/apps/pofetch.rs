use crate::poprint;

const OS: &'static str = "PopoenOS";
const VER: &'static str = "0.0.1";
const ASCII_ART: &str = r#"
 /$$$$$$$  /$$$$$$$   /$$$$$$   /$$$$$$ 
| $$__  $$| $$__  $$ /$$__  $$ /$$__  $$
| $$  \ $$| $$  \ $$| $$  \ $$| $$  \__/
| $$$$$$$/| $$$$$$$/| $$  | $$|  $$$$$$ 
| $$____/ | $$____/ | $$  | $$ \____  $$
| $$      | $$      | $$  | $$ /$$  \ $$
| $$      | $$      |  $$$$$$/|  $$$$$$/
|__/      |__/       \______/  \______/ 
"#;

pub fn pofetch() {

     poprint!("{}\n", ASCII_ART);
     poprint!("{} at version {}\n", OS, VER);
}
