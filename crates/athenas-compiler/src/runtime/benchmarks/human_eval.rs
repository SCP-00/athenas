use crate::runtime::benchmark::{
    BenchmarkMetadata, BenchmarkRunner, BenchmarkTask,
};

/// HumanEval-style programming benchmark.
///
/// Measures raw coding ability (L0) through structured code generation.
/// Tasks are function-level problems requiring specific implementations.
///
/// This is the reference implementation for the BenchmarkRunner trait.
/// Validation is structural (checks for function names, keywords, patterns)
/// rather than execution-based (does not require Python to run tests).
pub struct HumanEvalRunner;

impl HumanEvalRunner {
    pub fn new() -> Self {
        Self
    }

    fn tasks() -> Vec<BenchmarkTask> {
        vec![
            // --- Task 1: Binary Search ---
            BenchmarkTask {
                id: "HE-001".to_string(),
                description: "Implement binary search on a sorted array".to_string(),
                prompt: "Write a function `binary_search(arr, target)` that returns the index of target in a sorted array arr, or -1 if not found. The array is 0-indexed.".to_string(),
                required_elements: vec![
                    "def binary_search".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 2: FizzBuzz ---
            BenchmarkTask {
                id: "HE-002".to_string(),
                description: "Classic FizzBuzz".to_string(),
                prompt: "Write a function `fizzbuzz(n)` that returns a list of strings from 1 to n. For multiples of 3, use 'Fizz'. For multiples of 5, use 'Buzz'. For multiples of both, use 'FizzBuzz'.".to_string(),
                required_elements: vec![
                    "def fizzbuzz".to_string(),
                    "Fizz".to_string(),
                    "Buzz".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 3: Reverse String ---
            BenchmarkTask {
                id: "HE-003".to_string(),
                description: "Reverse a string".to_string(),
                prompt: "Write a function `reverse_string(s)` that returns the string reversed.".to_string(),
                required_elements: vec![
                    "def reverse_string".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 4: Palindrome Check ---
            BenchmarkTask {
                id: "HE-004".to_string(),
                description: "Check if a string is a palindrome".to_string(),
                prompt: "Write a function `is_palindrome(s)` that returns True if the string is a palindrome, ignoring case and non-alphanumeric characters.".to_string(),
                required_elements: vec![
                    "def is_palindrome".to_string(),
                    "return".to_string(),
                    "True".to_string(),
                    "False".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 5: Two Sum ---
            BenchmarkTask {
                id: "HE-005".to_string(),
                description: "Find two numbers that sum to target".to_string(),
                prompt: "Write a function `two_sum(nums, target)` that returns the indices of two numbers in nums that add up to target. Each input has exactly one solution. You may not use the same element twice.".to_string(),
                required_elements: vec![
                    "def two_sum".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 6: Fibonacci ---
            BenchmarkTask {
                id: "HE-006".to_string(),
                description: "Compute nth Fibonacci number".to_string(),
                prompt: "Write a function `fibonacci(n)` that returns the nth Fibonacci number (0-indexed: fib(0) = 0, fib(1) = 1).".to_string(),
                required_elements: vec![
                    "def fibonacci".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 7: Valid Parentheses ---
            BenchmarkTask {
                id: "HE-007".to_string(),
                description: "Check valid parentheses".to_string(),
                prompt: "Write a function `is_valid_parentheses(s)` that returns True if the string has valid parentheses, brackets, and braces: '()', '[]', '{}'. They must be properly nested.".to_string(),
                required_elements: vec![
                    "def is_valid_parentheses".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 8: Maximum Subarray ---
            BenchmarkTask {
                id: "HE-008".to_string(),
                description: "Kadane's algorithm — maximum subarray sum".to_string(),
                prompt: "Write a function `max_subarray(nums)` that returns the largest sum of any contiguous subarray using Kadane's algorithm.".to_string(),
                required_elements: vec![
                    "def max_subarray".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 9: Merge Sorted Arrays ---
            BenchmarkTask {
                id: "HE-009".to_string(),
                description: "Merge two sorted arrays".to_string(),
                prompt: "Write a function `merge_sorted(arr1, arr2)` that merges two sorted arrays into one sorted array.".to_string(),
                required_elements: vec![
                    "def merge_sorted".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 10: First Non-Repeating Character ---
            BenchmarkTask {
                id: "HE-010".to_string(),
                description: "First non-repeating character in a string".to_string(),
                prompt: "Write a function `first_unique_char(s)` that returns the index of the first non-repeating character in a string, or -1 if none exists.".to_string(),
                required_elements: vec![
                    "def first_unique_char".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 11: Linked List Cycle Detection ---
            BenchmarkTask {
                id: "HE-011".to_string(),
                description: "Detect cycle in a linked list".to_string(),
                prompt: "Write a function `has_cycle(head)` that returns True if the linked list has a cycle. Use Floyd's algorithm (two pointers). Assume a ListNode class with a 'next' attribute.".to_string(),
                required_elements: vec![
                    "def has_cycle".to_string(),
                    "return".to_string(),
                    "next".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 12: String Compression ---
            BenchmarkTask {
                id: "HE-012".to_string(),
                description: "Compress a string using counts".to_string(),
                prompt: "Write a function `compress_string(s)` that compresses a string using counts of repeated characters (e.g., 'aabcccccaaa' -> 'a2b1c5a3'). If the compressed string is not shorter, return the original.".to_string(),
                required_elements: vec![
                    "def compress_string".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 13: Anagrams ---
            BenchmarkTask {
                id: "HE-013".to_string(),
                description: "Check if two strings are anagrams".to_string(),
                prompt: "Write a function `are_anagrams(s1, s2)` that returns True if the two strings are anagrams of each other (same characters, different order). Case-insensitive.".to_string(),
                required_elements: vec![
                    "def are_anagrams".to_string(),
                    "return".to_string(),
                    "True".to_string(),
                    "False".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 14: Binary Tree Inorder Traversal ---
            BenchmarkTask {
                id: "HE-014".to_string(),
                description: "Inorder traversal of binary tree".to_string(),
                prompt: "Write a function `inorder_traversal(root)` that returns a list of values from an inorder traversal of a binary tree. TreeNode has 'val', 'left', 'right' attributes.".to_string(),
                required_elements: vec![
                    "def inorder_traversal".to_string(),
                    "return".to_string(),
                    "left".to_string(),
                    "right".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 15: Factorial ---
            BenchmarkTask {
                id: "HE-015".to_string(),
                description: "Compute factorial".to_string(),
                prompt: "Write a function `factorial(n)` that returns n! (n factorial). Handle n=0 (0! = 1).".to_string(),
                required_elements: vec![
                    "def factorial".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 16: Power of Two ---
            BenchmarkTask {
                id: "HE-016".to_string(),
                description: "Check if a number is a power of two".to_string(),
                prompt: "Write a function `is_power_of_two(n)` that returns True if n is a power of two, False otherwise. n is a positive integer.".to_string(),
                required_elements: vec![
                    "def is_power_of_two".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 17: Missing Number ---
            BenchmarkTask {
                id: "HE-017".to_string(),
                description: "Find missing number in sequence".to_string(),
                prompt: "Write a function `missing_number(nums)` that finds the missing number in an array containing n distinct numbers taken from 0, 1, 2, ..., n.".to_string(),
                required_elements: vec![
                    "def missing_number".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 18: Count Vowels ---
            BenchmarkTask {
                id: "HE-018".to_string(),
                description: "Count vowels in a string".to_string(),
                prompt: "Write a function `count_vowels(s)` that returns the number of vowels (a, e, i, o, u) in a string. Case-insensitive.".to_string(),
                required_elements: vec![
                    "def count_vowels".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 19: Remove Duplicates ---
            BenchmarkTask {
                id: "HE-019".to_string(),
                description: "Remove duplicates from sorted array".to_string(),
                prompt: "Write a function `remove_duplicates(nums)` that removes duplicates from a sorted array in-place and returns the new length. Do not allocate extra space.".to_string(),
                required_elements: vec![
                    "def remove_duplicates".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },

            // --- Task 20: Valid Anagram ---
            BenchmarkTask {
                id: "HE-020".to_string(),
                description: "Valid anagram (alternative implementation)".to_string(),
                prompt: "Write a function `valid_anagram(s, t)` that returns True if t is an anagram of s. Use character counting (not sorting).".to_string(),
                required_elements: vec![
                    "def valid_anagram".to_string(),
                    "return".to_string(),
                ],
                forbidden_elements: vec![
                    ".sort(".to_string(),
                    "sorted(".to_string(),
                ],
                validation_type: "code".to_string(),
                reference: None,
                language: Some("python".to_string()),
            },
        ]
    }
}

impl BenchmarkRunner for HumanEvalRunner {
    fn id(&self) -> &str {
        "human-eval"
    }

    fn metadata(&self) -> BenchmarkMetadata {
        BenchmarkMetadata {
            id: "human-eval".to_string(),
            name: "HumanEval-Style Coding".to_string(),
            description: "20 function-level programming tasks measuring raw coding ability, structural code generation, and algorithmic thinking.".to_string(),
            task_count: Self::tasks().len(),
            max_level: 3,
            languages: vec!["python".to_string()],
            requires_docker: false,
            requires_network: false,
            estimated_duration_minutes: 30,
            tier: "tier-0".to_string(),
        }
    }

    fn discover_tasks(&self) -> Result<Vec<BenchmarkTask>, String> {
        Ok(Self::tasks())
    }

    fn validate(&self, task: &BenchmarkTask, model_output: &str) -> Result<bool, String> {
        // Code-level validation: check that required elements are present
        for required in &task.required_elements {
            if !model_output.contains(required) {
                return Err(format!(
                    "Missing required element '{required}' in output for task {}",
                    task.id
                ));
            }
        }

        // Check forbidden elements
        for forbidden in &task.forbidden_elements {
            if model_output.contains(forbidden) {
                return Err(format!(
                    "Contains forbidden element '{forbidden}' in output for task {}",
                    task.id
                ));
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_eval_metadata() {
        let runner = HumanEvalRunner::new();
        let meta = runner.metadata();
        assert_eq!(meta.id, "human-eval");
        assert_eq!(meta.task_count, 20);
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn test_human_eval_discover_tasks() {
        let runner = HumanEvalRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        assert_eq!(tasks.len(), 20);
        assert_eq!(tasks[0].id, "HE-001");
        assert_eq!(tasks[0].language.as_deref(), Some("python"));
    }

    #[test]
    fn test_validate_passes() {
        let runner = HumanEvalRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let task = &tasks[0];

        // A valid solution should pass
        let valid_output = "def binary_search(arr, target):\n    left, right = 0, len(arr) - 1\n    while left <= right:\n        mid = (left + right) // 2\n        if arr[mid] == target:\n            return mid\n        elif arr[mid] < target:\n            left = mid + 1\n        else:\n            right = mid - 1\n    return -1";

        let result = runner.validate(task, valid_output);
        assert!(result.is_ok(), "Valid output should pass: {:?}", result.err());
    }

    #[test]
    fn test_validate_fails_missing_required() {
        let runner = HumanEvalRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let task = &tasks[0];

        // Missing the function name
        let invalid_output = "print('hello')";
        let result = runner.validate(task, invalid_output);
        assert!(result.is_err(), "Missing function name should fail");
    }

    #[test]
    fn test_validate_fails_forbidden() {
        let runner = HumanEvalRunner::new();
        let tasks = runner.discover_tasks().unwrap();

        // Task HE-020 forbids .sort() and sorted()
        let task = &tasks[19];
        let output_with_sort = "def valid_anagram(s, t):\n    return sorted(s) == sorted(t)";
        let result = runner.validate(task, output_with_sort);
        assert!(result.is_err(), "Using sorted() should fail for HE-020");
    }
}
