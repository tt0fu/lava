(x/5) - perceived difficulty

# Proper runtime profiling (2/5)

Add proper host and device timings which then can be analyzed to tune the performance.

# Moving away to slang from glsl (4?/5)

Vulkano slang support is still in early beta. Will have to wait until it improves.

# Adding custom shader support (6/5)

Would require having to do shader compilation, SPIR-V and slang reflection, dynamic shader type creation, manually managing struct offsets, etc.
