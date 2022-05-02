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

# 2022/04/02 Choosing rendering method
## Context
How to render SVG?
1. Use native svg renderer available for each platform
1. Use available SVG renderer
1. Build Custom SVG renderer

## Decision
Build Custom SVG renderer

using native svg renderer available for each platform would likely end up with less code, however considering multiple platforms may have different apis to update svg paths, each with different optimization, creating a binding for each seemed like a lot of work and also uninteresting from my perspective.

Using available SVG renderer
I considered lyon and tried out with their code but considering SVGs are generally unintended to animate, it seemed it would need to tesselate every frame which is heavy work.
I also considered resvg but it had no example of rendering svgs to a window (only rendering to pngs) and I wasn't too sure how to use it to render to a window nor was I interested enough to research, so I decided to not to use it.

Maybe there were other libraries I could consider, but most are for tesselation. Such tesselation library are often not intended for svgs to be dynamically reshaped, which means it could require tesselation for every reshape that occurs; Slightly concerning performance wise.


## Consequences
I will probably not spend too much time to make it a perfect svg renderer. Rather something rudimentary that only supports fills and strokes.

With the use of usvg library, fills and strokes shall be expressive enough for most UIs like text / curved shapes
The first version would not have any filters so shadows commonly used in Material UI are unavailable at first because shadows require gaussian blur.
