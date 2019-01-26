* adopt euclid as foundational points/geometry library
* collision detection
  * add bounding boxes to `Shape` trait
  * scene maintains AABB tree of shapes in the scene
  * use AABB tree to quickly find candidates for closer inspection
    * good overview:
      https://www.azurefromthetrenches.com/introductory-guide-to-aabb-tree-collision-detection/
  * use separating axis theorem to determine if it is actually a collision
    * have to do O(m*n) checks on each primitive shape
* clipping
  * Vatti is most general
  * Greiner-Hormann is faster, but convex only
