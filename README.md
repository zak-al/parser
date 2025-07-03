# [Work in progress] Parser: a simple, safe parser combinator library in Rust

This is an ongoing personal project. It is not yet thorougly tested, and therefore to be used with due diligence! + some fundamental changes can be made at any moment. So not really meant to be used.

A _parser_ is an instance of a class that implements the trait `Parser`. Parsers have a name and a method, `parse`, that takes an input (of type `&str`, for the moment) and returns an object of type `AbstractSyntaxTree` along with a reference to the remaining input upon success, and a `ParseError` upon failure (through a `Result` enum).

`AbstractSyntaxTree`s contain a tag (as a string, for the moment) meant to describe what was parsed, and either a string or a list of sub-`AbstractSyntaxTree`s.

The library provides elementary parsers, referred to as `atoms`. For example, the class `StringParser` allows to define parsers that recognise a particular string, and `AllWordsFromAlphabet` allows to define parsers that recognise arbitrarily long strings over a specified set of characters.

It also provides _wrappers_ around parsers. Under the hood, wrappers are nothing more than parsers that have as parameters other parsers. `Ignore` is a wrapper that fails if the wrapped parser fails, and accepts while discarding the resulting abstract syntax tree when the wrapped parser accepts -- this can be used when we want to check whether some pattern is indeed in the input but won't need to process the matched string after parsing.

`Brick`s are another kind of wrappers. They allow to combine arbitrarily many parsers in a versatile way. A brick consist of a list of parsers $(p_1, \mathellipsis, p_n)$ and maps $\{1, \mathellipsis, n\} \to \{1, \mathellipsis, n\}$ that describe what parser to apply next, knowing whether the previous parser accepted or rejected its input.
These maps can encode composition of parsers, as _when the n-th parser accepts, try the (n+1)th; when any parser fails, reject the input and when the last parser accepts, accept the input_. They can also encode disjunction, as _if the n-th parser fails, try the (n+1)-th; if any parser accepts, accept; if the last parser fails, fail_ and chains of inputs with separators as _if the input parser accepts, try to parse the separator; if the separator accepts, try to parse the input; if the separator fails, accept; and if the input parser fails, reject_.
These dynamics, along with some others, are built-in: we can just use the corresponding constructor (e.g. `Brick::make_linear` for composition, passing as parameter the list of parsers in the right order). Or we can define a basic brick with a list of parsers and then set for each parser what to do next in case of success and failure.

A formal documentation will come someday, in the meantime I hope the code is sufficiently clear well-commented to understand most details!
