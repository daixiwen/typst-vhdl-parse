// helper functions to look for comments in the code
use vhdl_lang::{Token, TokenAccess, TokenId};

/// Find the description (comment) associated with an object by inspecting
/// the comments attached to tokens in the token stream.
///
/// The vhdl_lang tokenizer attaches comments to tokens:
/// - `trailing`: a comment on the same line as the token, after it.
///   Same-line comments (e.g., `port_name : in std_logic; -- desc`) are attached
///   as `trailing` on the **semicolon** token at the end of the line.
/// - `leading`: comments on lines before the token. A solo comment on the line
///   immediately before the object declaration is attached as the last `leading`
///   comment on the **identifier** token.
///
/// if have_first_token is true, it means the token:id is the first on the line and
/// we can use the vhdl_lang crate's leading() method directly to get the
/// leading comments. If false it means we need to go through the tokens to find the
/// first on the line before calling leading()
///
/// If no description is found, returns None.
pub fn find_object_description(
    tokens: &[Token],
    object_token_id: TokenId,
    priority_trailing: bool,
    use_leading: bool,
) -> Option<String> {
    let object_token = tokens.index(object_token_id);
    let object_line = object_token.pos.range.start.line;

    let trailing = find_trailing_comment(tokens, object_line);
    let leading = if use_leading {
        find_leading_comment(object_token, object_line)
    } else {
        // look for a "trailing" comment on the line above
        if let Some(first_token_on_line) = find_first_token(tokens, object_line) {
            find_leading_comment(first_token_on_line, object_line)
        } else {
            None
        }
    };

    // apply priority
    if priority_trailing {
        return trailing.or(leading);
    } else {
        return leading.or(trailing);
    }
}

/// Scan the token list for an object on the same line that has a
///    trailing comment — this is the same-line comment description.
pub fn find_trailing_comment(tokens: &[Token], object_line: u32) -> Option<String> {
    for token in tokens.iter() {
        let token_line = token.pos.range.start.line;

        if token_line == object_line {
            if let Some(comments) = &token.comments {
                if let Some(trailing) = &comments.trailing {
                    return Some(trailing.value.trim().to_string());
                }
            }
        } else if token_line > object_line {
            // we got passed the line, don't need to continue
            break;
        }
    }
    return None;
}

/// Check the identifier token's leading comments for a solo comment
///    on the line immediately before the object
pub fn find_leading_comment(object_token: &Token, object_line: u32) -> Option<String> {
    if let Some(comments) = &object_token.comments {
        if let Some(leading) = comments.leading.last() {
            // A leading comment on the line just before the port
            if leading.range.end.line + 1 == object_line {
                return Some(leading.value.trim().to_string());
            }
        }
    }
    return None;
}

/// Scan the token list for an object on the same line that has a
///    trailing comment — this is the same-line comment description.
pub fn find_first_token(tokens: &[Token], object_line: u32) -> Option<&Token> {
    for token in tokens.iter() {
        let token_line = token.pos.range.start.line;

        if token_line == object_line {
            return Some(token);
        } else if token_line > object_line {
            // we got passed the line, don't need to continue
            break;
        }
    }
    return None;
}
