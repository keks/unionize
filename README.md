# Study of 2-3-Trees

- Immutable data structure / CoW
  - Updates return new root
  - this way we can't have parent links
    - they would require nodes to only be in a single tree
  - instead, have a tree traversal cursor that keeps track of the path
