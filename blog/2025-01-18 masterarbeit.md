# My Master's Thesis

## Fuzzing as Editor Feedback

You can [download my Master's thesis here](files/Masterarbeit.pdf).
This is the abstract:

> Fuzzing – testing code with random inputs to see if it crashes – is a well-researched technique.
> In this work, we explore how fuzzing enables new editor tooling.
> We integrate fuzzing into the compiler pipeline and language tooling of Martinaise, a custom programming language.
> Our prototype can fuzz individual functions and show results in Visual Studio Code.
> Unlike other methods for testing code, this requires no extra effort from the developer.
> Our tool shows crashing inputs next to the function signature.
> This shortens the feedback loop compared to traditional fuzzing, as edge cases are shown while the code is written, thereby preventing bugs.
> Our tool also displays example inputs, chosen based on code coverage.
> Examples that execute different parts of a function can help understand the function’s behavior without reading the implementation.
> Examples can be filtered to ones reaching the cursor position.
> We evaluate the quality and performance of our prototype and for which kind of code fuzzing can give useful examples.
> We discuss how our approach can be applied to other programming languages.
> Fuzzing has the potential to serve as the basis for Babylonian Programming, visualizations, debugging sessions, and other tools that benefit from concrete examples.
