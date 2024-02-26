//We store the tree in an 8-bit RGBA 3D texture called the indirection pool. Each "pixel" of the indirection pool is called a cell.




struct IndirectionPool {
    pool: texture_storage_3d<rgba8unorm, read_write>	
}


// The indirection pool is subdivided into indirection grids. An indirection grid is a cube of NxNxN cells (a 2x2x2 grid for an octree). 
// Each node of the tree is represented by an indirection grid. It corresponds to the array of pointers in the CPU implementation 
// described earlier.