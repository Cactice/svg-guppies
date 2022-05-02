# What is this
[What is ADR](https://github.com/joelparkerhenderson/architecture-decision-record#:~:text=Timestamp%20format-,What%20is%20an%20architecture%20decision%20record%3F,that%20addresses%20a%20significant%20requirement.)

---

# 2022/05/02 Windowing & Renderer dependency
## Context

The dependency direction between windowing and renderer can be configured in many ways

1. windowing <- renderer
1. windowing -> renderer
1. windowing -> contracts, renderer -> contracts

## Decision
windowing -> renderer

## Consequences
Swapping windowing would not be difficult compared to swapping renderer.
Swapping renderer on the other hand will be difficult because windowing relies directly on renderer for things like vertices.
This can be made easier by using another file dependency 'contract' which both depends on, but this would be too complicated for such an early phase of this library.


# 2022/05/02 Choosing Math library
## Context
Many math library exists for making vector operations easier with rust.
i.e. cgmath, nalgebra, glam

## Decision
Use glam

## Consequences
glam has less stars than cgmath or nalgebra.
I chose this library for the build time and at the moment I didn't need anything beyond vec2.

https://github.com/bitshifter/mathbench-rs
