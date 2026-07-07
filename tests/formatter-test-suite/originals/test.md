# OmniFormatter Test Suite — Markdown

## CASE 1: Trailing whitespace   
This line has trailing spaces.   
This line too.  
Normal line.

## CASE 2: Excessive blank lines




This content follows 4 blank lines (should be capped at 2).

## CASE 3: Headers — various levels

# H1 Header

## H2 Header

### H3 Header

#### H4 Header

##### H5 Header

## CASE 4: Code blocks (must not be touched)

```javascript
const   x='hello';
let   y  =  'world'   ;
```

```python
def foo(   x ,   y   ):
    return x+y
```

    indented code block (4 spaces — CommonMark code block)
    this must be preserved exactly

## CASE 5: Inline code

Use `const x = 1` for constant declarations.
The `   spaced   ` content must stay.

## CASE 6: Lists

- Item 1   
- Item 2 with trailing whitespace   
- Item 3

1. Ordered one
2. Ordered two   
3. Ordered three with trailing  

## CASE 7: Blockquotes

> This is a quote   
> With trailing whitespace   
> Normal line

## CASE 8: Tables (must not be touched)

| Column A | Column B | Column C |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
| Long     | Short    | Medium   |

## CASE 9: Links and images

[Link text](https://example.com)
![Alt text](https://example.com/image.png)

## CASE 10: Mixed content




Paragraph after many blank lines.

Another paragraph.



Yet another paragraph after two blank lines (should keep 2).

## CASE 11: Fenced code block toggle state

```
Not a language-tagged block.
The formatter opening fence toggles fence state.
```

Back to normal prose.

```bash
echo "inside fence"
```

Final line of the document.
