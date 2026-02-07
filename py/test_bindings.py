import polarisdb
import time

def test_rust_bindings():
    print("Testing PolarisDB Python bindings...")

    # Initialize index
    index = polarisdb.Index("cosine", 3)
    print("[OK] Index initialized")

    # Insert vectors
    vectors = [
        (1, [1.0, 0.0, 0.0]),
        (2, [0.0, 1.0, 0.0]),
        (3, [0.0, 0.0, 1.0]),
        (4, [1.0, 1.0, 0.0]), # Should be close to 1 and 2
    ]

    start = time.time()
    for id, vector in vectors:
        index.insert(id, vector)
    print(f"[OK] Inserted {len(vectors)} vectors in {(time.time() - start)*1000:.2f}ms")

    # Search
    query = [1.0, 0.1, 0.0]
    results = index.search(query, 2)
    
    print(f"[OK] Search results for {query}:")
    for id, dist in results:
        print(f"   ID: {id}, Distance: {dist:.4f}")

    assert results[0][0] == 1, "Expected ID 1 to be closest"
    # Collection (Persistent)
    print("\nTesting Persistent Collection...")
    import shutil
    import os
    if os.path.exists("./py_test_col"):
        shutil.rmtree("./py_test_col")

    col = polarisdb.Collection.open_or_create("./py_test_col", 3, "cosine")
    col.insert(1, [1.0, 0.0, 0.0])
    col.flush()
    print("[OK] Collection created and flushed")
    
    col2 = polarisdb.Collection.open_or_create("./py_test_col", 3, "cosine")
    res = col2.search([1.0, 0.0, 0.0], 1)
    assert len(res) == 1
    assert res[0][0] == 1
    print("[OK] Persistence verified")
    
    if os.path.exists("./py_test_col"):
        shutil.rmtree("./py_test_col")

    print("All checks passed!")

if __name__ == "__main__":
    test_rust_bindings()
