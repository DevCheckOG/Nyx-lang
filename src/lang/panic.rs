pub struct PanicHandler<'a> {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub source: Option<&'a str>,
    pub message: &'a str,
}

impl<'a> PanicHandler<'a> {
    pub fn new(
        line: Option<usize>,
        column: Option<usize>,
        source: Option<&'a str>,
        message: &'a str,
    ) -> PanicHandler<'a> {
        PanicHandler {
            line,
            column,
            source,
            message,
        }
    }

    pub fn panic(&self) {
        if self.line.is_none() && self.column.is_none() && self.source.is_none() {
            panic!("\n{}\n", self.message);
        } else if self.source.unwrap().is_empty() {
            panic!(
                "\n{} ({}:{})\n",
                self.message,
                self.line.unwrap(),
                self.column.unwrap()
            );
        }

        panic!(
            "\n{} ({}:{})\n\n-----> {} <-----\n",
            self.message,
            self.line.unwrap(),
            self.column.unwrap(),
            self.source.unwrap()
        );
    }
}
