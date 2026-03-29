#!/usr/bin/env python3
"""
Verification script for rst_queue installation

This script tests:
1. Module import
2. Basic functionality
3. API availability
"""

import sys
import traceback


def test_import():
    """Test basic import"""
    print("Testing import...")
    try:
        from rst_queue import AsyncQueue, ExecutionMode
        print("✓ Import successful")
        return True
    except ImportError as e:
        print(f"✗ Import failed: {e}")
        traceback.print_exc()
        return False


def test_queue_creation():
    """Test queue creation"""
    print("\nTesting queue creation...")
    try:
        from rst_queue import AsyncQueue, ExecutionMode
        
        # Sequential
        seq_queue = AsyncQueue(mode=ExecutionMode.SEQUENTIAL)
        assert seq_queue.get_mode() == 0
        print("✓ Sequential queue created")
        
        # Parallel
        par_queue = AsyncQueue(mode=ExecutionMode.PARALLEL)
        assert par_queue.get_mode() == 1
        print("✓ Parallel queue created")
        
        return True
    except Exception as e:
        print(f"✗ Queue creation failed: {e}")
        traceback.print_exc()
        return False


def test_push_and_stats():
    """Test push and stats"""
    print("\nTesting push and stats...")
    try:
        from rst_queue import AsyncQueue
        
        queue = AsyncQueue()
        queue.push(b"test data")
        
        assert queue.total_pushed() == 1
        print(f"✓ Pushed 1 item, total_pushed = {queue.total_pushed()}")
        
        stats = queue.get_stats()
        assert hasattr(stats, 'total_pushed')
        assert hasattr(stats, 'total_processed')
        assert hasattr(stats, 'active_workers')
        print(f"✓ Stats available: {stats}")
        
        return True
    except Exception as e:
        print(f"✗ Push/stats test failed: {e}")
        traceback.print_exc()
        return False


def test_api_methods():
    """Test API methods"""
    print("\nTesting API methods...")
    try:
        from rst_queue import AsyncQueue
        
        queue = AsyncQueue()
        
        # Test all methods exist
        methods = [
            'push',
            'get_mode',
            'set_mode',
            'total_pushed',
            'total_processed',
            'total_errors',
            'active_workers',
            'get_stats',
            'start',
        ]
        
        for method in methods:
            assert hasattr(queue, method), f"Missing method: {method}"
        
        print(f"✓ All API methods available ({len(methods)} methods)")
        return True
    except Exception as e:
        print(f"✗ API method test failed: {e}")
        traceback.print_exc()
        return False


def main():
    """Run all tests"""
    print("=" * 60)
    print("rst_queue Installation Verification")
    print("=" * 60)
    
    tests = [
        test_import,
        test_queue_creation,
        test_push_and_stats,
        test_api_methods,
    ]
    
    results = []
    for test in tests:
        try:
            result = test()
            results.append(result)
        except Exception as e:
            print(f"\n✗ Test crashed: {e}")
            traceback.print_exc()
            results.append(False)
    
    print("\n" + "=" * 60)
    passed = sum(results)
    total = len(results)
    print(f"Results: {passed}/{total} tests passed")
    print("=" * 60)
    
    if all(results):
        print("\n✓ Installation verified successfully!")
        return 0
    else:
        print("\n✗ Some tests failed. Please check the output above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
