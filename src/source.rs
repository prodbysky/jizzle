pub struct Source {
    src: Vec<char>,
    offset: usize,
}
impl Source {
    pub fn new(src: &str) -> Self {
        Self {
            offset: 0,
            src: src.chars().collect(),
        }
    }
    pub fn finished(&self) -> bool {
        self.src.len() <= self.offset
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn src(&self) -> &[char] {
        &self.src
    }

    pub fn as_string(&self) -> String {
        self.src.iter().collect()
    }

    pub fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.next();
        }
    }

    pub fn peek(&self) -> Option<&char> {
        self.src.get(self.offset)
    }
    pub fn next(&mut self) -> Option<&char> {
        let c = self.src.get(self.offset);
        self.offset += 1;
        c
    }
    pub fn get_position(&self, offset: usize) -> (usize, usize) {
        let line = self.src[0..offset].iter().filter(|&&c| c == '\n').count() + 1;
        let column = self.src[0..offset]
            .iter()
            .rev()
            .take_while(|&&c| c != '\n')
            .count()
            + 1;
        (line, column)
    }
    pub fn get_line(&self, line_number: usize) -> String {
        let mut current_line = 1;
        let mut line_start = 0;

        for (i, &c) in self.src.iter().enumerate() {
            if c == '\n' {
                current_line += 1;
                line_start = i + 1;
            }

            if current_line == line_number && (i + 1 == self.src.len() || self.src[i + 1] == '\n') {
                return self.src[line_start..=i].iter().collect();
            }
        }

        if current_line == line_number {
            return self.src[line_start..].iter().collect();
        }

        String::new()
    }
}
