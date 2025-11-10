mod formatting;
mod printing;

use crate::parser::ModelicaCST;

impl ModelicaCST {
    /// Return string containing formatted Modelica code represented by the CST.
    pub fn pretty_print(&self) -> String {
        let markers = formatting::format(self, None);
        printing::print(self, markers)
    }

    /// Return string containing formatted Modelica code with specified max line length.
    pub fn pretty_print_with_line_length(&self, max_line_length: usize) -> String {
        let markers = formatting::format(self, Some(max_line_length));
        printing::print(self, markers)
    }
}
