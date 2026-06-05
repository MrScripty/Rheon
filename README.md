# Rheon
Real-time lowish resolution 3D fluid physics for creating refrence images to drive AI image diffusion. You would use this to when you want positionaly acurite AI rendered fluid effects. When using AI rendering it is often not nessisary to provide high accuracy visual instruction to get good looking reasults, but it can be difficult to achive deliberate precision of object placment and distribution without providing a guiding image. Instead of using more traditional 3D simulation tools and plugins Rheon provides an speedy and simple to use simulation toolkit whos only concern is to be good enoough to pass guidance to AI while using minimal memory to maintain space for loaded weights. 

This repo will include a standalone demonstration GUI and a modular simulation framework that can be implemented into other apps. Idealy you would integrate Rheon into your own AI design tool as part of your seamless production pipeline.

# About the name
Derived from rheology (study of flowing/deforming matter, especially non-Newtonian liquids) + Greek rhein (to flow)
