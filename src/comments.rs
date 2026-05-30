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
/// If no description is found, returns None.
pub fn find_object_description(
    tokens: &[Token],
    object_token_id: TokenId,
    priority_trailing: bool,
) -> Option<String> {
    let object_token = tokens.index(object_token_id);
    let object_line = object_token.pos.range.start.line;

    let trailing = find_trailing_comment(tokens, object_line);
    let leading = find_leading_comment( object_token, object_line);

    // apply priority
    if priority_trailing {
        return trailing.or(leading);
    } else {
        return leading.or(trailing);
    }
}

pub fn find_trailing_comment(
    tokens: &[Token],
    object_line: u32,
) -> Option<String> {

    // Scan the token list for an object on the same line that has a
    //    trailing comment — this is the same-line comment description.
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

pub fn find_leading_comment(
    object_token: &Token,
    object_line: u32,
) -> Option<String> {
    // Check the identifier token's leading comments for a solo comment
    //    on the line immediately before the object
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
