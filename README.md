# gpu-gui

Using gpu to generate dynamic vector graphics for GUI purpose

# Design Goals

### Vector Graphics as View

Typically, GUI vector graphics designed on figma cannot be used directly as the GUI. 
Often, a developer reimplments the design in langugages like HTML so the interface can take interactions like click and text input. 

This two step process can be reduced if the View of MVC was strictly focused on showing and had no focus on interactions.
In such MVC, the View could be a .svg file. 
Combined with gpu-gui program which adds dynamic programability to svg files,
 features like click, text input, and responsiveness would be available.
 
### Layout requires no additional knowledge other than some basic math concepts.

This is in contrast to methods like CSS where arbitary concepts `flexbox` or `block` is cruicial.
CSS requires studying these concepts, rather than exposing the math behind it.
If concepts like `flexbox` and `block` are convinient, it should be provided similar to `std` libraries, just as a convience abstraction.
There'd be the benefit of allowing competitions among convinience abstractions, which would make inconvinient abstractions obsolete earlier than if it was provided as primary methods like CSS does.


### NICE TO HAVE: DDT (Design Driven Tests)

Designs created on figma can be considered a test case for a specific state.


# Minimal Test Cases

1. Center a rectangle 
1. Resize a rounded rectangle
1. Resize a text field with word break

