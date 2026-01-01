use super::*;

impl DslParser {
    pub(crate) fn parse_block(&mut self, kind: BlockKind) -> Result<AstNode, VMError> {
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(self.parse_error("' {' to start block"));
        }
        self.advance();

        let mut statements = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            statements.push(self.parse_statement()?);
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(VMError::UnexpectedEndOfInput);
        }
        self.advance();

        Ok(AstNode::Block { statements, kind })
    }

    pub(crate) fn parse_block_or_statement(&mut self) -> Result<AstNode, VMError> {
        if matches!(self.current_token, Token::LeftBrace) {
            self.parse_block(BlockKind::Regular)
        } else {
            self.parse_statement()
        }
    }
}
