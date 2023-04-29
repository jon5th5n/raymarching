# raymarching
a dynamic raymarcher written in rust and glsl with the help from macroquad just for getting the shader onto the screen

## Why?
I always found raymarching to be one of the coolest ways you can put 3D geometry onto a screen especially due to the ease of manipulating them through the combination of different functions.
This is not my first approach at something like this. This is (I think) my third try in a few years and it's definetly the furthest I came so far.
Spinning up a simple raymarcher on the CPU was never the problem but due to my lack of knowledge on shaders I always struggled with getting anywhere of an acceptable performance. My biggest barrier was always the communication from CPU to GPU since I wanted to have a dynamic experience.

## Dynamic
... means that the scene which is rendered to the screen can be controlled at runtime allowing you to move, add or remove objects just as you would like (not yet implemented).
This is achieved by passing a Texture, basically containing instruction for the GPU, to the shader where it is parsed to get all the instructions back. The process involves converting between u8's, u32's and f32's alot and traversing the tree structured instructions without recursion since it is not allowed in glsl which made things pretty challenging.

## GLSL
My knowledge on GLSL or on shaders in general was near zero before this projects and is still pretty much non existent so there are propably way better solutions to many of my problems that I just didn't notice. 
