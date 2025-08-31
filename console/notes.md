Coding-partner console is and abstraction of user interface with two main implementations - stdio, terminal.

1. The first one stdio is simple REPL interface.
    - placed in module 'stdio'
    - So user send command to stdin or response on the program request
      like yes/no, accept/reject, file name, or any information request by the program.
    - Program prints all to stdout
      without any terminal related formatting like color, alignment, bold/italic/underline text modificators.

2. The second is a terminal REPL interface like it implemented in the GEMINI, Claude Code, ...
    - placed in module 'terminal'
    - So Program is boarding areas on the terminal for the user input, for the program output. All choice are presented
      in output area by selecting available choices or with text input if it requires specific data like file name or any text input.
    - Every Request - Response cycle is boarded and request is differentiate by a color

3. there can be another implementations so as much functionality as possible must be reusable and be placed in module 'common'

Abstract interface presents next behaviour:

presented as an async trait in the module 'common' 

1. Get user request (prompting)
2. Start responding
3. Add text to respond
4. Add additional information like thinking process, command usage, internet searching, information about any other action
5. Query from user to make a choice from selected variants, Yes/No, Accept/Reject
6. Query from user to get any arbitrary text
7. Finish responding

General implementation includes tagging and command support as it implemented in other Coding Assistance like GEMINI

- Simbol / is prefixing command from the possible commands which is part of ConsoleInput enumeration
    + terminal implementation display command short help under the user input with limited set of variants,
      if there are not enough lines under the input line it scrolls terminal for needed lines and use this area.
    + terminal helps autocomplete command and presents
- Symbol @ is prefixing file path
    + terminal implementation helps with autocomplete file name and displays limited count of variants
      under the input area, if there are not enough lines under the input line it scrolls terminal for needed
      lines and use this area.
    + file can be part of the any text request and command.

stdio implementation of console does not have any information about terminal and available to work with redirected streams 
    to be used in tests and automation