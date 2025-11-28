# Markdown Edge Cases & Nested Structures

---

## Deeply Nested Lists

- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6
            - Level 7 (max nesting)

1. First
   1. Nested ordered
      - Mixed with unordered
        1. Back to ordered
           - [ ] Task in deep nest
             - Even deeper
               1. And ordered again

---

## Lists Inside Blockquotes

> - Item in quote
> - Another item
>   - Nested in quote
>     - Deeper still
>
> 1. Ordered in quote
> 2. Second item
>    - Mixed nesting

> > Nested quote with list:
> >
> > - Item 1
> > - Item 2
> >   > > > Triple nested quote
> >   > > >
> >   > > > - With a list item

---

## Blockquotes Inside Lists

- List item with quote:

  > This quote is inside a list item
  > It can span multiple lines

  Paragraph continues after quote.

- Another item
  > > Nested quote in list
  > >
  > > > Even deeper

1. Ordered with quote
   > Quote content
   >
   > - List in quote in list
   >   > Quote in list in quote in list

---

## Code Blocks in Lists

- Item with code:

  ```python
  def nested_function():
      return "code in list"
  ```

- Another item

  1. Nested ordered

     ```javascript
     const deeply = {
       nested: {
         code: true,
       },
     };
     ```

     > Quote after code in list
     >
     > ```rust
     > // Code in quote in list
     > fn main() {}
     > ```

---

## Tables in Lists

- Item with table:

  | Col A | Col B |
  | ----- | ----- |
  | 1     | 2     |

- Nested table:

  - Deeper:

    | X   | Y   | Z   |
    | --- | --- | --- |
    | a   | b   | c   |

---

## Complex Table Content

| Feature   | Example                                | Code       |
| --------- | -------------------------------------- | ---------- |
| **Bold**  | _Italic_                               | `inline`   |
| [Link](/) | ![img](https://via.placeholder.com/20) | ~~strike~~ |
| List:     | - a                                    | - b        |
| Quote:    | > text                                 | `code`     |
| Mixed     | **_bold italic_**                      | **_all_**  |

| Nested `code` | More **bold `code`** |
| ------------- | -------------------- |
| `a` + `b`     | **`styled code`**    |

---

## Links and Images Nested

[![**Bold alt text**](https://via.placeholder.com/100 "Nested title")](https://example.com "Link title")

[Link with `code` inside](https://example.com)

[**Bold link** with _italic_](https://example.com)

> ![Image in quote](https://via.placeholder.com/80)
>
> [Link in quote](https://example.com)

- ![Image in list](https://via.placeholder.com/60)
  - [![Linked image in nested list](https://via.placeholder.com/40)](https://example.com)

---

## Formatting Combinations

**_~~Bold italic strikethrough~~_**

**~~Bold strikethrough~~**

_~~Italic strikethrough~~_

<sup>**Bold superscript**</sup>

<sub>*Italic subscript*</sub>

<mark>**Bold highlighted**</mark>

<kbd>**Ctrl**</kbd> + <kbd>*Shift*</kbd> + <kbd>`K`</kbd>

---

## Code with Special Characters

```html
<div class="test" data-value="a && b || c">
  &lt;escaped&gt; &amp; entities
  <script>
    alert('XSS "test"');
  </script>
</div>
```

```javascript
const regex = /^[a-z]+\$\{.*\}$/gi;
const template = `
    Backticks: \`nested\`
    Dollar: $100
    Escape: \\n \\t
`;
const obj = { key: "value", [`dynamic_${id}`]: true };
```

```bash
echo "Nested \"quotes\" and 'single' and \`backticks\`"
cat <<'EOF'
    Heredoc with $VAR that won't expand
EOF
var="$(echo "command $(nested)")"
```

```python
string = """
Triple quoted with "double" and 'single'
And a literal \n newline escape
"""
f_string = f"Value: {obj['key']}"
raw = r"Raw string: \n stays literal"
```

---

## Escaping Edge Cases

\*\*Not bold\*\*

\`Not code\`

\[Not a link\](url)

\> Not a quote

\- Not a list

\| Not | a | table |

\\Backslash\\

\*Mixed **real bold** with escaped\*

---

## HTML Mixed with Markdown

<div>

**Bold inside div**

- List inside div
- Second item

</div>

<table>
<tr>
<td>

**Markdown in table cell**

- Works here
- Nested list

```python
code_in_html_table = True
```

</td>
<td>

> Quote in cell

</td>
</tr>
</table>

<details>
<summary>**Bold summary** with `code`</summary>

### Heading in details

- List item
  - Nested
    ```javascript
    const inDetails = true;
    ```

> Quote in details
>
> - List in quote in details

| Table | In  | Details |
| ----- | --- | ------- |
| a     | b   | c       |

</details>

---

## Footnotes with Complex Content

Text with footnote[^complex].

Another reference[^code].

And nested[^nested].

[^complex]: Footnote with **bold**, _italic_, and `code`.

    Second paragraph in footnote.

    - List in footnote
    - Second item

[^code]: Footnote with code:

    ```python
    def in_footnote():
        pass
    ```

[^nested]:
    > Quote in footnote
    >
    > - List in quote in footnote

---

## Task Lists Edge Cases

- [x] ~~Completed and struck~~
- [ ] **Bold incomplete**
- [x] Task with `code`
- [ ] Task with [link](https://example.com)
- [ ] Nested tasks:
  - [x] Sub-task done
  - [ ] Sub-task pending
    - [x] Deep sub-task
- [ ] Task with quote:
  > Quoted text in task
- [ ] Task with code block:
  ```
  code in task
  ```

---

## Alerts with Nested Content

> [!NOTE]
> Note with **formatting** and `code`
>
> - List in alert
> - Second item
>   ```python
>   code_in_alert = True
>   ```

> [!WARNING]
>
> > Nested quote in warning
> >
> > | Table | In  | Alert |
> > | ----- | --- | ----- |
> > | a     | b   | c     |

> [!IMPORTANT]
>
> 1. Ordered list in alert
> 2. Second item
>    - [x] Task in alert
>    - [ ] Another task

---

## Multi-line Table Cells (HTML)

<table>
<tr>
<th>Feature</th>
<th>Complex Content</th>
</tr>
<tr>
<td>Multi-line</td>
<td>

Line 1

Line 2

Line 3

</td>
</tr>
<tr>
<td>With Code</td>
<td>

```rust
fn complex() {
    println!("In table");
}
```

</td>
</tr>
<tr>
<td>Nested Structure</td>
<td>

- Item 1
  - Nested
    > Quote
    >
    > ```js
    > code();
    > ```

</td>
</tr>
</table>

---

## Reference Links Stress Test

[ref1][1] [ref2][2] [ref3][3] [same][1] [again][1]

[implicit][]

[case insensitive][CASE]

[1]: https://example.com/1 "Title 1"
[2]: https://example.com/2
[3]: https://example.com/3 "Title 3"
[implicit]: https://example.com/implicit
[CASE]: https://example.com/case

---

## Unicode and Special Characters

| Category | Examples        |
| -------- | --------------- |
| Emoji    | 🎉 🚀 💻 🔥 ⚡  |
| Math     | ∑ ∏ √ ∞ ≠ ≤ ≥   |
| Arrows   | → ← ↑ ↓ ↔ ⇒     |
| Greek    | α β γ δ ε θ λ π |
| Currency | $ € £ ¥ ₿       |
| Box      | ┌─┬─┐ │ │ └─┴─┘ |
| Dingbats | ✓ ✗ ★ ☆ ♠ ♣ ♥ ♦ |

```
ASCII art preserved:
    ┌───────────┐
    │  Box Art  │
    └───────────┘
```

---

## Zero-Width and Invisible

Normal​Word (zero-width space between)

Soft­hyphen (may break here)

Word with combining accent: é (e + ´)

---

## Edge Case URLs

[Parentheses](<https://en.wikipedia.org/wiki/Rust_(programming_language)>)

[Query params](https://example.com?a=1&b=2&c=3)

[Fragment](https://example.com/page#section-1)

[Complex](https://user:pass@example.com:8080/path?query=value#hash)

[Encoded](https://example.com/path%20with%20spaces)

<https://example.com/autolink>

---

_End of Edge Cases_
