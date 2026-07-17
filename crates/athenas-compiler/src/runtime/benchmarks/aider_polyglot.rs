use crate::runtime::benchmark::{resolve_validator, 
    BenchmarkMetadata, BenchmarkRunner, BenchmarkTask,
};

/// Aider Polyglot benchmark — multi-language code editing.
///
/// Based on Exercism-style exercises across 6 languages:
/// C++, Go, Java, JavaScript, Python, Rust.
///
/// This is the second BenchmarkRunner implementation and validates
/// the plugin architecture by proving a new runner can be added
/// without modifying the certification engine.
pub struct AiderPolyglotRunner;

impl AiderPolyglotRunner {
    pub fn new() -> Self {
        Self
    }

    fn tasks() -> Vec<BenchmarkTask> {
        let mut all = Vec::new();

        // =====================================================================
        // PYTHON (38 tasks: PY-001 to PY-038)
        // =====================================================================
        all.extend(vec![
            // PY-001: Hello World
            BenchmarkTask {
                id: "PY-001".into(),
                description: "Write Hello World function".into(),
                prompt: "Write a function `hello(name)` that returns a greeting string. If name is empty or None, return 'Hello, World!' otherwise return 'Hello, {name}!'.".into(),
                required_elements: vec!["def hello".into(), "return".into(), "Hello, World!".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-002: Leap year
            BenchmarkTask {
                id: "PY-002".into(),
                description: "Determine leap year".into(),
                prompt: "Write a function `is_leap_year(year)` that returns True if the year is a leap year. A leap year is divisible by 4, except if divisible by 100, unless also divisible by 400.".into(),
                required_elements: vec!["def is_leap_year".into(), "return".into(), "True".into(), "False".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-003: RNA transcription
            BenchmarkTask {
                id: "PY-003".into(),
                description: "RNA transcription".into(),
                prompt: "Write a function `to_rna(dna_strand)` that returns the RNA complement of a DNA strand. G->C, C->G, T->A, A->U.".into(),
                required_elements: vec!["def to_rna".into(), "return".into(), ".translate".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-004: Scrabble score
            BenchmarkTask {
                id: "PY-004".into(),
                description: "Scrabble word score".into(),
                prompt: "Write a function `score(word)` that returns the Scrabble score for a word. A=1, E=1, I=1, O=1, U=1, L=1, N=1, R=1, S=1, T=1, D=2, G=2, B=3, C=3, M=3, P=3, F=4, H=4, V=4, W=4, Y=4, K=5, J=8, X=8, Q=10, Z=10.".into(),
                required_elements: vec!["def score".into(), "return".into(), "dict".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-005: Acronym
            BenchmarkTask {
                id: "PY-005".into(),
                description: "Acronym generator".into(),
                prompt: "Write a function `abbreviate(phrase)` that returns the acronym for a phrase. Example: 'Portable Network Graphics' -> 'PNG'. Handle hyphenated words.".into(),
                required_elements: vec!["def abbreviate".into(), "return".into(), ".upper".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-006: Raindrops
            BenchmarkTask {
                id: "PY-006".into(),
                description: "Raindrops number translation".into(),
                prompt: "Write a function `convert(number)` that returns 'Pling' if divisible by 3, 'Plang' if by 5, 'Plong' if by 7. Combine for multiple factors. Return number as string if no factors.".into(),
                required_elements: vec!["def convert".into(), "return".into(), "Pling".into(), "Plang".into(), "Plong".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-007: Difference of squares
            BenchmarkTask {
                id: "PY-007".into(),
                description: "Difference of squares".into(),
                prompt: "Write functions `square_of_sum(n)` and `sum_of_squares(n)` and `difference(n)` that compute the difference between the square of the sum and the sum of the squares of the first n natural numbers.".into(),
                required_elements: vec!["def square_of_sum".into(), "def sum_of_squares".into(), "def difference".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-008: Resistor color duo
            BenchmarkTask {
                id: "PY-008".into(),
                description: "Resistor color code decoding".into(),
                prompt: "Write a function `value(colors)` that decodes two resistor colors into a number. Colors: black=0, brown=1, red=2, orange=3, yellow=4, green=5, blue=6, violet=7, grey=8, white=9.".into(),
                required_elements: vec!["def value".into(), "return".into(), "list".into(), "index".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-009: Isogram
            BenchmarkTask {
                id: "PY-009".into(),
                description: "Check if word is isogram".into(),
                prompt: "Write a function `is_isogram(string)` that returns True if the string is an isogram (no repeating letters, ignoring case and hyphens).".into(),
                required_elements: vec!["def is_isogram".into(), "return".into(), "set".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-010: Pangram
            BenchmarkTask {
                id: "PY-010".into(),
                description: "Check pangram".into(),
                prompt: "Write a function `is_pangram(sentence)` that returns True if the sentence uses every letter of the alphabet at least once. Case-insensitive.".into(),
                required_elements: vec!["def is_pangram".into(), "return".into(), "set".into(), "string".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-011: Perfect numbers
            BenchmarkTask {
                id: "PY-011".into(),
                description: "Classify perfect numbers".into(),
                prompt: "Write a function `classify(number)` that returns 'perfect', 'abundant', or 'deficient' based on the sum of its proper divisors. Perfect = sum == number, Abundant = sum > number, Deficient = sum < number.".into(),
                required_elements: vec!["def classify".into(), "return".into(), "perfect".into(), "abundant".into(), "deficient".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-012: Sum of multiples
            BenchmarkTask {
                id: "PY-012".into(),
                description: "Sum of multiples".into(),
                prompt: "Write a function `sum_of_multiples(limit, factors)` that returns the sum of all unique multiples of the given factors below the limit. If factors is empty, use [3, 5].".into(),
                required_elements: vec!["def sum_of_multiples".into(), "return".into(), "set".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-013: Grade school
            BenchmarkTask {
                id: "PY-013".into(),
                description: "Grade school roster".into(),
                prompt: "Write a class `School` with methods: `add_student(name, grade)`, `roster()` (returns all students sorted), and `grade(n)` (returns students in grade n sorted).".into(),
                required_elements: vec!["class School".into(), "def add_student".into(), "def roster".into(), "def grade".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-014: Robot name
            BenchmarkTask {
                id: "PY-014".into(),
                description: "Robot name generator".into(),
                prompt: "Write a class `Robot` that generates random names like 'AB123' (two uppercase letters + three digits). `reset()` generates a new name. Names must be unique.".into(),
                required_elements: vec!["class Robot".into(), "def __init__".into(), "def reset".into(), "random".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-015: Clock
            BenchmarkTask {
                id: "PY-015".into(),
                description: "Clock addition/subtraction".into(),
                prompt: "Write a class `Clock` that stores time (hour, minute) and supports `__add__(minutes)` and `__sub__(minutes)` methods. Overload `__eq__` and `__repr__`. Wrap around 24 hours.".into(),
                required_elements: vec!["class Clock".into(), "def __add__".into(), "def __sub__".into(), "def __eq__".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-016: Matrix
            BenchmarkTask {
                id: "PY-016".into(),
                description: "Matrix rows and columns".into(),
                prompt: "Write a class `Matrix` that takes a string like '1 2\\n3 4' and has `row(index)` and `column(index)` methods (1-indexed).".into(),
                required_elements: vec!["class Matrix".into(), "def row".into(), "def column".into(), "__init__".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-017: Sieve
            BenchmarkTask {
                id: "PY-017".into(),
                description: "Sieve of Eratosthenes".into(),
                prompt: "Write a function `primes(limit)` that returns all prime numbers up to the given limit using the Sieve of Eratosthenes.".into(),
                required_elements: vec!["def primes".into(), "return".into(), "list".into(), "range".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-018: Luhn
            BenchmarkTask {
                id: "PY-018".into(),
                description: "Luhn algorithm validation".into(),
                prompt: "Write a function `is_valid(card_number)` that validates a number using the Luhn algorithm. Count digits, double every second from the right (subtract 9 if >9), check if sum is divisible by 10.".into(),
                required_elements: vec!["def is_valid".into(), "return".into(), "True".into(), "False".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-019: Grains
            BenchmarkTask {
                id: "PY-019".into(),
                description: "Chessboard grain calculation".into(),
                prompt: "Write functions `square(number)` (returns grains on that square, 2^(n-1)) and `total()` (returns total grains on all 64 squares). Raise ValueError for invalid squares.".into(),
                required_elements: vec!["def square".into(), "def total".into(), "ValueError".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-020: Pascals triangle
            BenchmarkTask {
                id: "PY-020".into(),
                description: "Pascal's triangle".into(),
                prompt: "Write a function `rows(n)` that returns the first n rows of Pascal's triangle as a list of lists. Each row is built from the previous row.".into(),
                required_elements: vec!["def rows".into(), "return".into(), "list".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-021: Phone number
            BenchmarkTask {
                id: "PY-021".into(),
                description: "Phone number cleaning".into(),
                prompt: "Write a function `clean(phone_number)` that cleans and validates a phone number (NANP format). Strip non-digits, check length (10 or 11 with 1 as country code), validate area code (2-9) and exchange code (2-9).".into(),
                required_elements: vec!["def clean".into(), "return".into(), "ValueError".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-022: Anagram
            BenchmarkTask {
                id: "PY-022".into(),
                description: "Find anagrams".into(),
                prompt: "Write a function `find_anagrams(word, candidates)` that returns a list of candidate words that are anagrams of the target word. An anagram must be the same length, use the same letters, and not be the same word.".into(),
                required_elements: vec!["def find_anagrams".into(), "return".into(), "sorted".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-023: Allergies
            BenchmarkTask {
                id: "PY-023".into(),
                description: "Allergy test scoring".into(),
                prompt: "Write a class `Allergies` that takes a score and has methods: `allergic_to(item)` (bool) and `lst` (property returning list). Scores: eggs=1, peanuts=2, shellfish=4, strawberries=8, tomatoes=16, chocolate=32, pollen=64, cats=128.".into(),
                required_elements: vec!["class Allergies".into(), "def allergic_to".into(), "lst".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-024: Queen attack
            BenchmarkTask {
                id: "PY-024".into(),
                description: "Queen attack calculator".into(),
                prompt: "Write a function `can_attack(white_queen, black_queen)` that returns True if two queens on a chess board can attack each other. Positions are (row, col) tuples. Queens can attack on same row, column, or diagonal.".into(),
                required_elements: vec!["def can_attack".into(), "return".into(), "abs".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-025: Simple cipher
            BenchmarkTask {
                id: "PY-025".into(),
                description: "Simple substitution cipher".into(),
                prompt: "Write a class `Cipher` that implements a simple substitution cipher. `encode(plain)` shifts each letter by a key. `decode(cipher)` reverses. Handle wrapping around the alphabet. Preserve case only for letters.".into(),
                required_elements: vec!["class Cipher".into(), "def encode".into(), "def decode".into(), "ord".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-026: OCR numbers
            BenchmarkTask {
                id: "PY-026".into(),
                description: "OCR digit recognition".into(),
                prompt: "Write a function `convert(input_grid)` that converts 3x4 character grids into numbers. Each digit is 3 rows high and 4 columns wide. Digits: use standard 7-segment patterns. Return '?' for unrecognizable.".into(),
                required_elements: vec!["def convert".into(), "return".into(), "dict".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-027: Roman numerals
            BenchmarkTask {
                id: "PY-027".into(),
                description: "Roman numeral conversion".into(),
                prompt: "Write a function `roman(number)` that converts an integer (1-3999) to Roman numerals. Use standard subtractive notation (IV=4, IX=9, XL=40, XC=90, CD=400, CM=900).".into(),
                required_elements: vec!["def roman".into(), "return".into(), "dict".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-028: Bowling
            BenchmarkTask {
                id: "PY-028".into(),
                description: "Bowling score calculator".into(),
                prompt: "Write a class `BowlingGame` with `roll(pins)` and `score()` methods. Handle strikes (10+next 2 rolls), spares (10+next roll), and the 10th frame bonus rules.".into(),
                required_elements: vec!["class BowlingGame".into(), "def roll".into(), "def score".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-029: Rectangles
            BenchmarkTask {
                id: "PY-029".into(),
                description: "Count rectangles in ASCII grid".into(),
                prompt: "Write a function `rectangles(strings)` that counts rectangles formed by '+' corners, '-' horizontal edges, and '|' vertical edges in an ASCII grid.".into(),
                required_elements: vec!["def rectangles".into(), "return".into(), "for".into(), "range".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-030: Twelve days
            BenchmarkTask {
                id: "PY-030".into(),
                description: "Twelve Days of Christmas lyrics".into(),
                prompt: "Write a function `verse(n)` that returns the nth verse and `verses(start, end)` that returns verses from start to end. Use the traditional 'Twelve Days of Christmas' lyrics.".into(),
                required_elements: vec!["def verse".into(), "def verses".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-031: Poker
            BenchmarkTask {
                id: "PY-031".into(),
                description: "Poker hand ranking".into(),
                prompt: "Write a function `best_hands(hands)` that returns the best poker hand(s) from a list of hands. Each hand is a string like '5H 5C 6S 7S KD'. Handle ties correctly. Recognize all standard hands (high card to straight flush).".into(),
                required_elements: vec!["def best_hands".into(), "return".into(), "sorted".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-032: Palindrome products
            BenchmarkTask {
                id: "PY-032".into(),
                description: "Find palindrome products".into(),
                prompt: "Write functions `smallest_palindrome(max_factor, min_factor)` and `largest_palindrome(max_factor, min_factor)` that find the smallest/largest palindrome that is a product of two numbers within the range.".into(),
                required_elements: vec!["def smallest_palindrome".into(), "def largest_palindrome".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-033: List Ops
            BenchmarkTask {
                id: "PY-033".into(),
                description: "Functional list operations".into(),
                prompt: "Write functions `append(l1, l2)`, `concat(lists)`, `filter(clause, l)`, `length(l)`, `map(fn, l)`, `foldl(fn, initial, l)`, `foldr(fn, initial, l)`, `reverse(l)` — all using recursion, no built-in iteration.".into(),
                required_elements: vec!["def append".into(), "def concat".into(), "def filter".into(), "def length".into(), "def map".into(), "def foldl".into(), "def foldr".into(), "def reverse".into()],
                forbidden_elements: vec!["for".into(), "while".into()],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-034: Zebra Puzzle
            BenchmarkTask {
                id: "PY-034".into(),
                description: "Einstein's zebra puzzle solver".into(),
                prompt: "Write a function `solve()` that solves the zebra puzzle (Einstein's riddle) using constraint propagation. Return a dict mapping nationalities to their house attributes (color, drink, smoke, pet). Use any logical deduction approach.".into(),
                required_elements: vec!["def solve".into(), "return".into(), "dict".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-035: Yacht
            BenchmarkTask {
                id: "PY-035".into(),
                description: "Yacht dice game scoring".into(),
                prompt: "Write a function `score(dice, category)` that scores a yacht dice game roll. Categories: ones through sixes, full house, four of a kind, little straight, big straight, choice, yacht (all five equal). Dice is a list of 5 integers 1-6.".into(),
                required_elements: vec!["def score".into(), "return".into(), "sorted".into(), "set".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-036: Space Age
            BenchmarkTask {
                id: "PY-036".into(),
                description: "Earth years conversion".into(),
                prompt: "Write a class `SpaceAge` that takes age in seconds and provides properties for each planet: `mercury`, `venus`, `earth`, `mars`, `jupiter`, `saturn`, `neptune`, `uranus`. Earth year = 31557600 seconds. Orbital ratios: Mercury=0.2408467, Venus=0.61519726, Mars=1.8808158, Jupiter=11.862615, Saturn=29.447498, Uranus=84.016846, Neptune=164.79132.".into(),
                required_elements: vec!["class SpaceAge".into(), "__init__".into(), "def earth".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-037: Circular buffer
            BenchmarkTask {
                id: "PY-037".into(),
                description: "Circular buffer implementation".into(),
                prompt: "Write a class `CircularBuffer` with `read()` and `write(value)` and `overwrite(value)` and `clear()` methods. `read()` raises `BufferEmptyException`, `write()` raises `BufferFullException` if full. `overwrite()` replaces oldest data.".into(),
                required_elements: vec!["class CircularBuffer".into(), "def read".into(), "def write".into(), "def overwrite".into(), "def clear".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
            // PY-038: House
            BenchmarkTask {
                id: "PY-038".into(),
                description: "House that Jack built".into(),
                prompt: "Write a function `recite(start, end)` that returns verses from 'The House That Jack Built' nursery rhyme. Each verse builds on the previous one recursively.".into(),
                required_elements: vec!["def recite".into(), "return".into(), "list".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("python".into()),
            },
        ]);

        // =====================================================================
        // GO (38 tasks: GO-001 to GO-038)
        // =====================================================================
        all.extend(vec![
            BenchmarkTask {
                id: "GO-001".into(),
                description: "Hello World".into(),
                prompt: "Write a Go package `greeting` with function `Hello(name string) string` that returns a greeting. If name is empty, return 'Hello, World!'.".into(),
                required_elements: vec!["package greeting".into(), "func Hello".into(), "string".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-002".into(),
                description: "Two-fer".into(),
                prompt: "Write a function `ShareWith(name string) string` that returns 'One for {name}, one for me.' If name is empty, use 'you'.".into(),
                required_elements: vec!["func ShareWith".into(), "string".into(), "return".into(), "fmt".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-003".into(),
                description: "Hamming distance".into(),
                prompt: "Write a function `Distance(a, b string) (int, error)` that returns Hamming distance between two DNA strands. Return error if strands have different lengths.".into(),
                required_elements: vec!["func Distance".into(), "string".into(), "int".into(), "error".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-004".into(),
                description: "Raindrops".into(),
                prompt: "Write a function `Convert(number int) string` that returns 'Pling' if divisible by 3, 'Plang' if by 5, 'Plong' if by 7, or the number as string.".into(),
                required_elements: vec!["func Convert".into(), "int".into(), "string".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-005".into(),
                description: "Scrabble score".into(),
                prompt: "Write a function `Score(word string) int` that returns Scrabble score for a word. Use a mapping of letters to values.".into(),
                required_elements: vec!["func Score".into(), "string".into(), "int".into(), "map".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-006".into(),
                description: "Isogram checker".into(),
                prompt: "Write a function `IsIsogram(word string) bool` that returns true if word is an isogram (no repeating letters, ignoring case and hyphens).".into(),
                required_elements: vec!["func IsIsogram".into(), "string".into(), "bool".into(), "return".into(), "map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-007".into(),
                description: "Difference of squares".into(),
                prompt: "Write functions: `SquareOfSum(n int) int` and `SumOfSquares(n int) int` and `Difference(n int) int`.".into(),
                required_elements: vec!["func SquareOfSum".into(), "func SumOfSquares".into(), "func Difference".into(), "int".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-008".into(),
                description: "Gigasecond anniversary".into(),
                prompt: "Write a function `AddGigasecond(t time.Time) time.Time` that adds 1,000,000,000 seconds to the given time.".into(),
                required_elements: vec!["func AddGigasecond".into(), "time.Time".into(), "time".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-009".into(),
                description: "RNA transcription".into(),
                prompt: "Write a function `ToRNA(dna string) string` that returns RNA complement. Use strings.Builder or a map for transcription.".into(),
                required_elements: vec!["func ToRNA".into(), "string".into(), "strings".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-010".into(),
                description: "Acronym generator".into(),
                prompt: "Write a function `Abbreviate(s string) string` that returns acronym, handling hyphens and underscores.".into(),
                required_elements: vec!["func Abbreviate".into(), "string".into(), "strings".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-011".into(),
                description: "Collatz conjecture".into(),
                prompt: "Write a function `CollatzConjecture(n int) (int, error)` that returns number of steps to reach 1. If n <= 0, return error.".into(),
                required_elements: vec!["func CollatzConjecture".into(), "int".into(), "error".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-012".into(),
                description: "Luhn validation".into(),
                prompt: "Write a function `Valid(id string) bool` that validates a number using the Luhn algorithm.".into(),
                required_elements: vec!["func Valid".into(), "string".into(), "bool".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-013".into(),
                description: "Palindrome products".into(),
                prompt: "Write functions `Smallest(min, max int) (int, [][2]int, error)` and `Largest(min, max int) (int, [][2]int, error)` that find palindrome products.".into(),
                required_elements: vec!["func Smallest".into(), "func Largest".into(), "int".into(), "error".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-014".into(),
                description: "Binary search".into(),
                prompt: "Write a function `SearchInts(slice []int, key int) int` that returns index of key in sorted slice, or -1 if not found.".into(),
                required_elements: vec!["func SearchInts".into(), "[]int".into(), "int".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-015".into(),
                description: "Anagram finder".into(),
                prompt: "Write a function `DetectAnagrams(subject string, candidates []string) []string` that returns anagram matches.".into(),
                required_elements: vec!["func DetectAnagrams".into(), "[]string".into(), "return".into(), "sort".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-016".into(),
                description: "Etl (transform legacy data)".into(),
                prompt: "Write a function `Transform(in map[int][]string) map[string]int` that transforms scrabble scores from old format to new format.".into(),
                required_elements: vec!["func Transform".into(), "map".into(), "string".into(), "int".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-017".into(),
                description: "Clock struct with methods".into(),
                prompt: "Write a `Clock` struct with `New(h, m int) Clock`, `Add(minutes int) Clock`, `Subtract(minutes int) Clock`, and `String() string` methods. 24-hour clock that wraps around.".into(),
                required_elements: vec!["type Clock struct".into(), "func New".into(), "func Add".into(), "func Subtract".into(), "func String".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-018".into(),
                description: "Matrix operations".into(),
                prompt: "Write a `Matrix` type with `New(s string) (*Matrix, error)`, `Rows() [][]int`, `Cols() [][]int` methods. Parse newline-separated rows of space-separated integers.".into(),
                required_elements: vec!["type Matrix".into(), "func New".into(), "func Rows".into(), "func Cols".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-019".into(),
                description: "Sieve of Eratosthenes".into(),
                prompt: "Write a function `Sieve(limit int) []int` that returns all primes up to limit using the sieve algorithm.".into(),
                required_elements: vec!["func Sieve".into(), "int".into(), "[]int".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-020".into(),
                description: "Parallel letter frequency".into(),
                prompt: "Write a function `Frequency(texts []string) map[rune]int` that computes letter frequency in parallel using goroutines and channels.".into(),
                required_elements: vec!["func Frequency".into(), "map[rune]int".into(), "go".into(), "chan".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-021".into(),
                description: "Word count".into(),
                prompt: "Write a function `WordCount(phrase string) map[string]int` that counts word occurrences, handling contractions and punctuation.".into(),
                required_elements: vec!["func WordCount".into(), "map[string]int".into(), "strings".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-022".into(),
                description: "Difference calculator".into(),
                prompt: "Write a function `Difference(s1, s2 []string) ([]string, []string, []string)` that returns added, removed, and unchanged items between two slices. Treat as set differences.".into(),
                required_elements: vec!["func Difference".into(), "[]string".into(), "map".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-023".into(),
                description: "Robot name struct".into(),
                prompt: "Write a `Robot` struct and `Name() string` and `Reset()` methods. Generate unique random names like 'AB123'.".into(),
                required_elements: vec!["type Robot struct".into(), "func Name".into(), "func Reset".into(), "math/rand".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-024".into(),
                description: "Grade school".into(),
                prompt: "Write a `School` struct with `Add(student string, grade int)`, `Grade(level int) []string`, and `Enrollment() map[int][]string` methods.".into(),
                required_elements: vec!["type School struct".into(), "func Add".into(), "func Grade".into(), "func Enrollment".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-025".into(),
                description: "LinkedList implementation".into(),
                prompt: "Write a function `NewList(elements ...interface{}) *List` and methods `Next()`, `Prev()`, `First()`, `Last()`, `Push(v interface{})`, `Pop() (interface{}, error)` for a doubly linked list.".into(),
                required_elements: vec!["type List struct".into(), "type Node struct".into(), "func NewList".into(), "func Next".into(), "func Push".into(), "interface{}".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-026".into(),
                description: "Binary search tree".into(),
                prompt: "Write functions `BstInsert(tree interface{}, value int) interface{}` and `InOrder(tree interface{}) []int` that implement a binary search tree with inorder traversal.".into(),
                required_elements: vec!["type BST".into(), "func BstInsert".into(), "func InOrder".into(), "[]int".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-027".into(),
                description: "Pangram checker".into(),
                prompt: "Write a function `IsPangram(s string) bool` that returns true if the sentence uses every letter of the alphabet.".into(),
                required_elements: vec!["func IsPangram".into(), "string".into(), "bool".into(), "unicode".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-028".into(),
                description: "Proverb generator".into(),
                prompt: "Write a function `Proverb(rhyme []string) []string` that returns a proverb from a list of words: 'For want of a {word} the {next} was lost.' and final line 'And all for the want of a {first}.'.".into(),
                required_elements: vec!["func Proverb".into(), "[]string".into(), "fmt".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-029".into(),
                description: "Series extraction".into(),
                prompt: "Write a function `All(n int, s string) []string` that returns all consecutive substrings of length n from string s.".into(),
                required_elements: vec!["func All".into(), "int".into(), "string".into(), "[]string".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-030".into(),
                description: "Crypto square".into(),
                prompt: "Write a function `Encode(plain string) string` that implements crypto square encoding: normalize, find rectangle dimensions, read down columns.".into(),
                required_elements: vec!["func Encode".into(), "string".into(), "return".into(), "rune".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-031".into(),
                description: "LED clock display".into(),
                prompt: "Write a function `DisplayTime(h, m int) string` that returns a 4-digit 7-segment LED display representation of a digital clock time.".into(),
                required_elements: vec!["func DisplayTime".into(), "int".into(), "string".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-032".into(),
                description: "Allergies".into(),
                prompt: "Write functions `AllergicTo(score int, substance string) bool` and `Allergies(score int) []string` using bitmask: eggs=1, peanuts=2, shellfish=4, etc.".into(),
                required_elements: vec!["func AllergicTo".into(), "func Allergies".into(), "int".into(), "bool".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-033".into(),
                description: "Queen attack".into(),
                prompt: "Write a function `CanAttack(w, b [2]int) bool` that returns true if two queens on a chess board can attack each other (same row, column, or diagonal).".into(),
                required_elements: vec!["func CanAttack".into(), "[2]int".into(), "bool".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-034".into(),
                description: "Roman numerals".into(),
                prompt: "Write a function `ToRomanNumeral(n int) (string, error)` that converts an integer (1-3999) to Roman numerals.".into(),
                required_elements: vec!["func ToRomanNumeral".into(), "int".into(), "string".into(), "error".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-035".into(),
                description: "Pascals triangle".into(),
                prompt: "Write a function `GenerateTriangle(n int) [][]int` that returns the first n rows of Pascal's triangle.".into(),
                required_elements: vec!["func GenerateTriangle".into(), "int".into(), "[][]int".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-036".into(),
                description: "Circular buffer".into(),
                prompt: "Write a `Buffer` struct with `NewBuffer(capacity int) *Buffer`, `ReadByte() (byte, error)`, `WriteByte(b byte) error`, `Overwrite(b byte)` methods.".into(),
                required_elements: vec!["type Buffer struct".into(), "func NewBuffer".into(), "func ReadByte".into(), "func WriteByte".into(), "func Overwrite".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-037".into(),
                description: "Bottle song".into(),
                prompt: "Write a function `Song() string` that returns the lyrics to '99 Bottles of Beer'. Handle pluralization correctly.".into(),
                required_elements: vec!["func Song".into(), "string".into(), "return".into(), "fmt".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
            BenchmarkTask {
                id: "GO-038".into(),
                description: "Tournament tally".into(),
                prompt: "Write a function `Tally(input string) string` that parses match results (W/L/D) and returns a formatted league table with wins, losses, draws, points.".into(),
                required_elements: vec!["func Tally".into(), "string".into(), "strings".into(), "sort".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("go".into()),
            },
        ]);

        // =====================================================================
        // RUST (38 tasks: RS-001 to RS-038)
        // =====================================================================
        all.extend(vec![
            BenchmarkTask {
                id: "RS-001".into(),
                description: "Hello World".into(),
                prompt: "Write a function `hello(name: Option<&str>) -> String` that returns a greeting. Return 'Hello, World!' for None or empty, otherwise 'Hello, {name}!'.".into(),
                required_elements: vec!["fn hello".into(), "Option".into(), "&str".into(), "String".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-002".into(),
                description: "Leap year".into(),
                prompt: "Write a function `is_leap_year(year: u64) -> bool` that returns true if year is a leap year.".into(),
                required_elements: vec!["fn is_leap_year".into(), "u64".into(), "bool".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-003".into(),
                description: "Reverse string".into(),
                prompt: "Write a function `reverse(input: &str) -> String` that returns the input string reversed. Use `.chars().rev().collect()`.".into(),
                required_elements: vec!["fn reverse".into(), "&str".into(), "String".into(), ".rev()".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-004".into(),
                description: "Gigasecond".into(),
                prompt: "Write a function `after(start: NaiveDateTime) -> NaiveDateTime` that returns the date one gigasecond (10^9 seconds) after the given time. Use chrono crate.".into(),
                required_elements: vec!["fn after".into(), "NaiveDateTime".into(), "chrono".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-005".into(),
                description: "Nucleotide count".into(),
                prompt: "Write a function `nucleotide_counts(dna: &str) -> Result<HashMap<char, usize>>` that counts occurrences of A, C, G, T in a DNA string.".into(),
                required_elements: vec!["fn nucleotide_counts".into(), "HashMap".into(), "Result".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-006".into(),
                description: "Raindrops".into(),
                prompt: "Write a function `raindrops(n: u32) -> String` that returns Pling/Plang/Plong based on divisibility by 3, 5, 7.".into(),
                required_elements: vec!["fn raindrops".into(), "u32".into(), "String".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-007".into(),
                description: "Variable-length quantity".into(),
                prompt: "Write functions `to_bytes(values: &[u32]) -> Vec<u8>` and `from_bytes(bytes: &[u8]) -> Result<Vec<u32>>` that encode/decode variable-length quantities for LEB128.".into(),
                required_elements: vec!["fn to_bytes".into(), "fn from_bytes".into(), "Vec<u8>".into(), "Result".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-008".into(),
                description: "Proverb".into(),
                prompt: "Write a function `proverb(strings: &[&str]) -> Vec<String>` that generates a proverb from a list of words.".into(),
                required_elements: vec!["fn proverb".into(), "&[&str]".into(), "Vec<String>".into(), "windows".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-009".into(),
                description: "Diffie-Hellman key exchange".into(),
                prompt: "Write a `PrivateKey(p: u64) -> u64`, `PublicKey(p: u64, g: u64, private: u64) -> u64`, and `Secret(p: u64, public: u64, private: u64) -> u64` for Diffie-Hellman key exchange.".into(),
                required_elements: vec!["fn private_key".into(), "fn public_key".into(), "fn secret".into(), "u64".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-010".into(),
                description: "ETL transformation".into(),
                prompt: "Write a function `transform(legacy: &HashMap<i32, Vec<String>>) -> HashMap<String, i32>` that transforms the old Scrabble scoring format to the new one.".into(),
                required_elements: vec!["fn transform".into(), "HashMap".into(), "Vec<String>".into(), "collect".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-011".into(),
                description: "Clock struct".into(),
                prompt: "Write a `Clock` struct with `new(hours: i32, minutes: i32) -> Clock`, add/subtract methods returning Clock, and `Display` impl. 24-hour wrap-around.".into(),
                required_elements: vec!["struct Clock".into(), "fn new".into(), "impl Display".into(), "impl Add".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-012".into(),
                description: "Grep clone".into(),
                prompt: "Write a function `grep(pattern: &str, files: &[&str]) -> Vec<String>` that mimics grep: search lines in each file. Include filename when multiple files.".into(),
                required_elements: vec!["fn grep".into(), "&str".into(), "Vec<String>".into(), "lines".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-013".into(),
                description: "Poker hand ranking".into(),
                prompt: "Write a `Hand` struct with a `new(hand: &str) -> Hand` and `rank(&self) -> Rank` that ranks poker hands (high card through straight flush).".into(),
                required_elements: vec!["struct Hand".into(), "fn new".into(), "fn rank".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-014".into(),
                description: "OCR digits".into(),
                prompt: "Write a function `convert(input: &str) -> Result<String, String>` that converts 3x4 OCR grids to numbers.".into(),
                required_elements: vec!["fn convert".into(), "&str".into(), "Result".into(), "lines".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-015".into(),
                description: "Bowling game".into(),
                prompt: "Write a `BowlingGame` struct with `roll(pins: u16) -> Result<()>` and `score() -> Result<u16>` methods implementing ten-pin bowling scoring.".into(),
                required_elements: vec!["struct BowlingGame".into(), "fn roll".into(), "fn score".into(), "Result".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-016".into(),
                description: "Rectangle counting".into(),
                prompt: "Write a function `count(lines: &[&str]) -> usize` that counts rectangles in an ASCII grid of '+', '-', '|', and spaces.".into(),
                required_elements: vec!["fn count".into(), "&[&str]".into(), "usize".into(), "chars".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-017".into(),
                description: "Zipper (tree zipper)".into(),
                prompt: "Write a `Zipper<T>` that represents a focus position in a binary tree with `from_tree`, `left`, `right`, `up`, `set_value`, `to_tree` methods. Use Box for child nodes.".into(),
                required_elements: vec!["struct Zipper".into(), "fn from_tree".into(), "fn left".into(), "fn right".into(), "Box".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-018".into(),
                description: "Binary search tree".into(),
                prompt: "Write `BinarySearchTree<T>` with `new(value: T)`, `insert(value: T)`, and `into_sorted_vec() -> Vec<T>` methods using owned nodes and Box.".into(),
                required_elements: vec!["struct BinarySearchTree".into(), "fn new".into(), "fn insert".into(), "fn into_sorted_vec".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-019".into(),
                description: "Matrix".into(),
                prompt: "Write a `Matrix` struct with `new(s: &str) -> Self`, `row(n: usize) -> Vec<u32>`, and `column(n: usize) -> Vec<u32>` methods. 1-indexed.".into(),
                required_elements: vec!["struct Matrix".into(), "fn new".into(), "fn row".into(), "fn column".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-020".into(),
                description: "Luhn from trait".into(),
                prompt: "Implement a `Luhn` trait with a `valid_luhn(&self) -> bool` method. Implement for `&str`, `String`, `u8`, `u16`, `u32`, `u64`, `usize`. Code validation using Luhn algorithm.".into(),
                required_elements: vec!["trait Luhn".into(), "fn valid_luhn".into(), "impl Luhn for".into(), "&str".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-021".into(),
                description: "Parallel letter frequency".into(),
                prompt: "Write a function `frequency(texts: &[&str], workers: usize) -> HashMap<char, usize>` that computes letter frequency using multiple threads. Use std::sync::mpsc or Arc+Mutex.".into(),
                required_elements: vec!["fn frequency".into(), "HashMap".into(), "Arc".into(), "Mutex".into(), "thread".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-022".into(),
                description: "Robot simulator".into(),
                prompt: "Write a RobotSimulator that handles `new` (pos, dir), `instructions(s: &str)`, and position queries. Directions: North, East, South, West. Instructions: 'A' advance, 'L' left, 'R' right.".into(),
                required_elements: vec!["struct Robot".into(), "fn new".into(), "fn instructions".into(), "#[derive".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-023".into(),
                description: "ISBN verifier".into(),
                prompt: "Write a function `is_valid_isbn(isbn: &str) -> bool` that validates ISBN-10 numbers. Check digit can be 'X' for 10.".into(),
                required_elements: vec!["fn is_valid_isbn".into(), "&str".into(), "bool".into(), "chars".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-024".into(),
                description: "Pascals triangle".into(),
                prompt: "Write a function `rows(n: u32) -> Vec<Vec<u32>>` that returns the first n rows of Pascal's triangle.".into(),
                required_elements: vec!["fn rows".into(), "u32".into(), "Vec<Vec<u32>>".into(), "windows".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-025".into(),
                description: "Simple cipher".into(),
                prompt: "Write a `Cipher` struct with `new(key: &str) -> Result<Cipher>` and `encode(plain: &str) -> String` and `decode(cipher: &str) -> String` for a substitution cipher.".into(),
                required_elements: vec!["struct Cipher".into(), "fn encode".into(), "fn decode".into(), "Result".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-026".into(),
                description: "Allergies".into(),
                prompt: "Write an `Allergies` struct with `new(score: u32) -> Self` and `is_allergic_to(item: &str) -> bool` and `allergies(&self) -> Vec<String>`. Use bitmask with match.".into(),
                required_elements: vec!["struct Allergies".into(), "fn new".into(), "fn is_allergic_to".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-027".into(),
                description: "Rail fence cipher".into(),
                prompt: "Write functions `encode(plain: &str, rails: u32) -> String` and `decode(cipher: &str, rails: u32) -> String` for the rail fence cipher.".into(),
                required_elements: vec!["fn encode".into(), "fn decode".into(), "String".into(), "chars".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-028".into(),
                description: "Minesweeper".into(),
                prompt: "Write a function `annotate(board: &[&str]) -> Vec<String>` that adds mine counts to a Minesweeper board. '*' is a mine, spaces get counts of adjacent mines.".into(),
                required_elements: vec!["fn annotate".into(), "&[&str]".into(), "Vec<String>".into(), "chars".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-029".into(),
                description: "Circular buffer".into(),
                prompt: "Write a `CircularBuffer<T>` with `new(capacity: usize)`, `read() -> Result<T>`, `write(value: T) -> Result<()>`, `overwrite(value: T)`, and `clear()` methods.".into(),
                required_elements: vec!["struct CircularBuffer".into(), "fn new".into(), "fn read".into(), "fn write".into(), "fn overwrite".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-030".into(),
                description: "Macro calculator".into(),
                prompt: "Write a function `calculate_macros(weight_kg: f64, goal: &str) -> HashMap<&str, f64>` that calculates recommended macros (protein, fat, carbs) based on weight and goal (lose/maintain/gain).".into(),
                required_elements: vec!["fn calculate_macros".into(), "HashMap".into(), "f64".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-031".into(),
                description: "Prime factors".into(),
                prompt: "Write a function `prime_factors(n: u64) -> Vec<u64>` that returns the prime factors of n in ascending order.".into(),
                required_elements: vec!["fn prime_factors".into(), "u64".into(), "Vec<u64>".into(), "while".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-032".into(),
                description: "Saddle points".into(),
                prompt: "Write a function `find_saddle_points(input: &[Vec<u64>]) -> Vec<(usize, usize)>` that finds saddle points in a matrix (largest in row, smallest in column).".into(),
                required_elements: vec!["fn find_saddle_points".into(), "&[Vec<u64>]".into(), "Vec<(usize, usize)>".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-033".into(),
                description: "Nth prime".into(),
                prompt: "Write a function `nth_prime(n: u32) -> Result<u64>` that returns the nth prime number. Return error for n=0.".into(),
                required_elements: vec!["fn nth_prime".into(), "u32".into(), "Result".into(), "u64".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-034".into(),
                description: "Accumulate".into(),
                prompt: "Write a function `map_function<T, U>(values: &[T], f: fn(&T) -> U) -> Vec<U>` that applies a function to each element, returning a new collection. Do NOT use .map(). Use iteration or recursion.".into(),
                required_elements: vec!["fn map_function".into(), "Vec".into(), "for".into(), "push".into()],
                forbidden_elements: vec![".map(".into()],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-035".into(),
                description: "Bob (lackadaisical teenager)".into(),
                prompt: "Write a function `reply(message: &str) -> &str` that returns Bob's responses: 'Sure.' for questions, 'Whoa, chill out!' for YELLING, 'Calm down, I know what I'm doing!' for yelled questions, 'Fine. Be that way!' for silence, 'Whatever.' for everything else.".into(),
                required_elements: vec!["fn reply".into(), "&str".into(), "match".into(), "trim".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-036".into(),
                description: "Space age".into(),
                prompt: "Write a struct `SpaceAge` with `new(seconds: f64)` and methods for each planet. Earth year = 31557600 seconds.".into(),
                required_elements: vec!["struct SpaceAge".into(), "fn new".into(), "fn on_earth".into(), "f64".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-037".into(),
                description: "Custom set".into(),
                prompt: "Write a `CustomSet<T>` with `new`, `insert`, `contains`, `is_empty`, `is_subset`, `is_disjoint`, `difference`, `union`, `intersection`, and `from_slice` methods.".into(),
                required_elements: vec!["struct CustomSet".into(), "fn new".into(), "fn contains".into(), "fn union".into(), "fn intersection".into(), "fn difference".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
            BenchmarkTask {
                id: "RS-038".into(),
                description: "Diamond pattern".into(),
                prompt: "Write a function `diamond(letter: char) -> Vec<String>` that returns a diamond pattern starting with 'A' at top and expanding symmetrically to the given letter.".into(),
                required_elements: vec!["fn diamond".into(), "char".into(), "Vec<String>".into(), "range".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("rust".into()),
            },
        ]);

        // =====================================================================
        // JAVASCRIPT (38 tasks: JS-001 to JS-038)
        // =====================================================================
        all.extend(vec![
            BenchmarkTask {
                id: "JS-001".into(),
                description: "Hello World".into(),
                prompt: "Export a function `hello(name)` that returns a greeting. If name is empty or undefined, return 'Hello, World!'.".into(),
                required_elements: vec!["export".into(), "function hello".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-002".into(),
                description: "Two-fer".into(),
                prompt: "Export a function `twoFer(name)` that returns 'One for {name}, one for me.'. Use 'you' if name is empty.".into(),
                required_elements: vec!["export".into(), "function twoFer".into(), "return".into(), "template".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-003".into(),
                description: "Leap year".into(),
                prompt: "Export a function `isLeapYear(year)` that returns true if year is a leap year.".into(),
                required_elements: vec!["export".into(), "function isLeapYear".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-004".into(),
                description: "Scrabble score".into(),
                prompt: "Export a function `score(word)` that returns Scrabble score for a word. Use an object as letter-value map.".into(),
                required_elements: vec!["export".into(), "function score".into(), "return".into(), "reduce".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-005".into(),
                description: "RNA transcription".into(),
                prompt: "Export a function `toRna(dna)` that returns RNA complement. Use an object as a complement map.".into(),
                required_elements: vec!["export".into(), "function toRna".into(), "return".into(), "split".into(), "map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-006".into(),
                description: "Resistor color".into(),
                prompt: "Export a function `colorCode(color)` that returns resistor value. Export `COLORS` array with all colors in order.".into(),
                required_elements: vec!["export".into(), "function colorCode".into(), "COLORS".into(), "indexOf".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-007".into(),
                description: "Acronym".into(),
                prompt: "Export a function `parse(phrase)` that returns acronym, handling hyphens.".into(),
                required_elements: vec!["export".into(), "function parse".into(), "return".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-008".into(),
                description: "Pangram".into(),
                prompt: "Export a function `isPangram(string)` that checks if string uses every letter of the alphabet.".into(),
                required_elements: vec!["export".into(), "function isPangram".into(), "return".into(), "every".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-009".into(),
                description: "Raindrops".into(),
                prompt: "Export a function `convert(number)` that returns 'Pling'/'Plang'/'Plong' string. Use a functional approach with array of factor-word pairs.".into(),
                required_elements: vec!["export".into(), "function convert".into(), "return".into(), "filter".into(), "map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-010".into(),
                description: "Diffie-Hellman key".into(),
                prompt: "Export functions `privateKey(p)`, `publicKey(p, g, private)`, and `secret(p, public, private)` for Diffie-Hellman key exchange. Use BigInt for large numbers.".into(),
                required_elements: vec!["export".into(), "function privateKey".into(), "function publicKey".into(), "function secret".into(), "BigInt".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-011".into(),
                description: "Anagram".into(),
                prompt: "Export a function `findAnagrams(word, candidates)` that returns array of anagrams. Use sorted-character comparison.".into(),
                required_elements: vec!["export".into(), "function findAnagrams".into(), "return".into(), "filter".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-012".into(),
                description: "Clock class".into(),
                prompt: "Export a `Clock` class with `constructor(h, m)`, `add(minutes)`, `subtract(minutes)` (returning new Clock), `equals(other)`, and `toString()` methods. 24-hour format.".into(),
                required_elements: vec!["export".into(), "class Clock".into(), "constructor".into(), "add".into(), "subtract".into(), "toString".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-013".into(),
                description: "Robot name".into(),
                prompt: "Export a `Robot` class with `constructor()` (generates random name 'AB123') and `reset()` method. Names must be unique across all instances.".into(),
                required_elements: vec!["export".into(), "class Robot".into(), "constructor".into(), "reset".into(), "Set".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-014".into(),
                description: "Matrix".into(),
                prompt: "Export a `Matrix` class with `constructor(string)` that parses a newline-separated string of spaceseparated numbers. Methods: `row(n)` and `column(n)` (1-indexed).".into(),
                required_elements: vec!["export".into(), "class Matrix".into(), "constructor".into(), "row".into(), "column".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-015".into(),
                description: "Pascals triangle".into(),
                prompt: "Export a function `rows(n)` that returns array of arrays representing Pascal's triangle.".into(),
                required_elements: vec!["export".into(), "function rows".into(), "return".into(), "Array.from".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-016".into(),
                description: "Word count".into(),
                prompt: "Export a function `countWords(phrase)` that returns an object mapping words to their counts. Handle contractions and punctuation.".into(),
                required_elements: vec!["export".into(), "function countWords".into(), "return".into(), "reduce".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-017".into(),
                description: "Phone number".into(),
                prompt: "Export a function `clean(number)` that cleans and validates a NANP phone number. Throw error for invalid numbers.".into(),
                required_elements: vec!["export".into(), "function clean".into(), "throw".into(), "replace".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-018".into(),
                description: "Grade school".into(),
                prompt: "Export a `School` class with `add(name, grade)`, `roster()` (returns all students sorted by grade then name), and `grade(n)` methods.".into(),
                required_elements: vec!["export".into(), "class School".into(), "add".into(), "roster".into(), "grade".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-019".into(),
                description: "Space age".into(),
                prompt: "Export a function `age(planet, seconds)` that returns the age on a given planet (earth years). Use an orbital periods object.".into(),
                required_elements: vec!["export".into(), "function age".into(), "return".into(), "const".into(), "orbitalPeriods".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-020".into(),
                description: "High score board".into(),
                prompt: "Export functions `createScoreBoard()`, `addPlayer(board, player, score)`, `removePlayer(board, player)`, `updateScore(board, player, score)`, `applyBonus(board, player, bonus)` that manage a high score board (object of player names to scores).".into(),
                required_elements: vec!["export".into(), "function createScoreBoard".into(), "function addPlayer".into(), "function removePlayer".into(), "function updateScore".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-021".into(),
                description: "Annalyn's infiltration".into(),
                prompt: "Export functions: `canFastAttack(knightIsAwake)`, `canSpy(knightIsAwake, archerIsAwake, prisonerIsAwake)`, `canSignalPrisoner(archerIsAwake, prisonerIsAwake)`, `canFreePrisoner(knightIsAwake, archerIsAwake, prisonerIsAwake, petDogIsPresent)` using boolean logic.".into(),
                required_elements: vec!["export".into(), "function canFastAttack".into(), "function canSpy".into(), "function canSignalPrisoner".into(), "function canFreePrisoner".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-022".into(),
                description: "Elyses enchantedments (array)".into(),
                prompt: "Export functions: `getFirstCard(deck)`, `getSecondCard(deck)`, `swapTopTwoCards(deck)`, `discardTopCard(deck)`, `insertFaceCards(deck)` that manipulate arrays of cards.".into(),
                required_elements: vec!["export".into(), "function getFirstCard".into(), "function getSecondCard".into(), "function swapTopTwoCards".into(), "function insertFaceCards".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-023".into(),
                description: "Lucky numbers (array transform)".into(),
                prompt: "Export functions: `transform(arr)` (multiply by 2), `filterLucky(arr)` (filter 7-containing nums), `sum(arr)` (reduce), `isPalindrome(arr)` (check symmetry), `duplicate(arr)` (map to 2-elt arrays). Use functional methods (map, filter, reduce).".into(),
                required_elements: vec!["export".into(), "function transform".into(), "function filterLucky".into(), "function sum".into(), "function isPalindrome".into(), "function duplicate".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-024".into(),
                description: "Fruit string sorting".into(),
                prompt: "Export a function `sortFruit(fruitArray)` that takes an array of mixed fruit strings like 'apple10' and 'apple2' and sorts them naturally (apple2 before apple10). Use regex and custom comparison.".into(),
                required_elements: vec!["export".into(), "function sortFruit".into(), "return".into(), "sort".into(), "match".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-025".into(),
                description: "Mixed juices (promises)".into(),
                prompt: "Export functions using Promises/async: `timeToMixJuice(name)` (returns promise resolving with time), `limesToCut(wedgesNeeded, limes)` (calculates limes needed), `remainingOrders(timeLeft, orders)` (removes completed orders based on time).".into(),
                required_elements: vec!["export".into(), "function timeToMixJuice".into(), "function limesToCut".into(), "function remainingOrders".into(), "async".into(), "Promise".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-026".into(),
                description: "Vehicle purchase".into(),
                prompt: "Export functions: `needsLicense(kind)` (returns bool for 'car'/'truck'), `chooseVehicle(option1, option2)` (returns the better one alphabetically), `calculateResellPrice(originalPrice, age)` (50% if <3yrs, 30% if >10yrs, else 70%).".into(),
                required_elements: vec!["export".into(), "function needsLicense".into(), "function chooseVehicle".into(), "function calculateResellPrice".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-027".into(),
                description: "Bob conversation".into(),
                prompt: "Export a function `hey(message)` that returns Bob's response based on message characteristics (question, yelling, silence, etc.).".into(),
                required_elements: vec!["export".into(), "function hey".into(), "return".into(), "trim".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-028".into(),
                description: "List flattening".into(),
                prompt: "Export a function `flatten(list)` that flattens a nested array of any depth, filtering out null/undefined values.".into(),
                required_elements: vec!["export".into(), "function flatten".into(), "return".into(), "reduce".into(), "Array.isArray".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-029".into(),
                description: "Minesweeper".into(),
                prompt: "Export a function `annotate(board)` that adds mine counts to a Minesweeper board. '*' is mine, spaces get count of adjacent mines.".into(),
                required_elements: vec!["export".into(), "function annotate".into(), "return".into(), "map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-030".into(),
                description: "Roman numerals".into(),
                prompt: "Export a function `toRoman(num)` that converts an integer (1-3999) to Roman numerals using subtractive notation.".into(),
                required_elements: vec!["export".into(), "function toRoman".into(), "return".into(), "reduce".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-031".into(),
                description: "Binary search".into(),
                prompt: "Export a function `binarySearch(sortedArray, target)` that returns index of target or -1 using iterative binary search.".into(),
                required_elements: vec!["export".into(), "function binarySearch".into(), "return".into(), "while".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-032".into(),
                description: "Sieve of Eratosthenes".into(),
                prompt: "Export a function `primes(limit)` that returns all primes up to limit using the sieve algorithm.".into(),
                required_elements: vec!["export".into(), "function primes".into(), "return".into(), "Array.from".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-033".into(),
                description: "Allergies".into(),
                prompt: "Export a class `Allergies` with `constructor(score)` and `list()` and `allergicTo(item)` methods. Use bitmask: eggs=1, peanuts=2, shellfish=4, etc.".into(),
                required_elements: vec!["export".into(), "class Allergies".into(), "constructor".into(), "list".into(), "allergicTo".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-034".into(),
                description: "Isogram checker".into(),
                prompt: "Export a function `isIsogram(word)` that returns true if the word is an isogram (no repeating letters).".into(),
                required_elements: vec!["export".into(), "function isIsogram".into(), "return".into(), "Set".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-035".into(),
                description: "Proverb".into(),
                prompt: "Export a function `proverb(words)` that returns array of proverb verses: 'For want of a {word} the {next} was lost.' + final line.".into(),
                required_elements: vec!["export".into(), "function proverb".into(), "return".into(), "map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-036".into(),
                description: "Custom set".into(),
                prompt: "Export a `CustomSet` class with `constructor(values)`, `empty()`, `contains(value)`, `add(value)`, `subset(other)`, `disjoint(other)`, `eql(other)`, `union(other)`, `intersection(other)`, `difference(other)` methods.".into(),
                required_elements: vec!["export".into(), "class CustomSet".into(), "constructor".into(), "contains".into(), "subset".into(), "union".into(), "intersection".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-037".into(),
                description: "Circular buffer".into(),
                prompt: "Export a class `CircularBuffer` with `constructor(capacity)`, `read()`, `write(value)`, `forceWrite(value)`, `clear()` methods. Throw BufferFullError on full write, BufferEmptyError on empty read.".into(),
                required_elements: vec!["export".into(), "class CircularBuffer".into(), "constructor".into(), "read".into(), "write".into(), "forceWrite".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
            BenchmarkTask {
                id: "JS-038".into(),
                description: "Series".into(),
                prompt: "Export a function `slice(series, sliceLength)` that returns all consecutive substrings of given length. Throw error if sliceLength > series.length.".into(),
                required_elements: vec!["export".into(), "function slice".into(), "return".into(), "map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("javascript".into()),
            },
        ]);

        // =====================================================================
        // JAVA (38 tasks: JV-001 to JV-038)
        // =====================================================================
        all.extend(vec![
            BenchmarkTask {
                id: "JV-001".into(),
                description: "Hello World".into(),
                prompt: "Write a public class `Greeter` with a static method `hello(String name)` that returns a greeting string. Use 'World' if name is null or empty.".into(),
                required_elements: vec!["public class Greeter".into(), "public static".into(), "String".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-002".into(),
                description: "Two-fer".into(),
                prompt: "Write a public class `Twofer` with a static method `twofer(String name)` returning 'One for {name}, one for me.'.".into(),
                required_elements: vec!["public class Twofer".into(), "public static".into(), "String".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-003".into(),
                description: "Leap year".into(),
                prompt: "Write a class `Leap` with a static method `isLeapYear(int year)` that returns boolean.".into(),
                required_elements: vec!["class Leap".into(), "static".into(), "boolean".into(), "isLeapYear".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-004".into(),
                description: "RNA transcription".into(),
                prompt: "Write a class `RnaTranscription` with a static method `transcribe(String dna)` that returns RNA complement. Use StringBuilder.".into(),
                required_elements: vec!["class RnaTranscription".into(), "static".into(), "String".into(), "StringBuilder".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-005".into(),
                description: "Resistor color".into(),
                prompt: "Write an enum `ResistorColor` with colors (BLACK through WHITE) and a static method `colorCode(String color)` returning int value. Use ordinal().".into(),
                required_elements: vec!["enum ResistorColor".into(), "colorCode".into(), "static".into(), "ordinal".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-006".into(),
                description: "Hamming distance".into(),
                prompt: "Write a class `Hamming` with a constructor taking two strands and a `getHammingDistance()` method that returns int. Throw IllegalArgumentException if lengths differ.".into(),
                required_elements: vec!["class Hamming".into(), "getHammingDistance".into(), "IllegalArgumentException".into(), "charAt".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-007".into(),
                description: "Scrabble score".into(),
                prompt: "Write a class `Scrabble` with a constructor taking a word and a `getScore()` method. Use a Map of letter values.".into(),
                required_elements: vec!["class Scrabble".into(), "getScore".into(), "Map".into(), "HashMap".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-008".into(),
                description: "Acronym".into(),
                prompt: "Write a class `Acronym` with a static method `generate(String phrase)` that returns the acronym. Handle hyphens and spaces.".into(),
                required_elements: vec!["class Acronym".into(), "static".into(), "String".into(), "split".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-009".into(),
                description: "Pangram checker".into(),
                prompt: "Write a class `PangramChecker` with a method `isPangram(String sentence)` that checks for all 26 letters. Use Set<Character>.".into(),
                required_elements: vec!["class PangramChecker".into(), "isPangram".into(), "Set<Character>".into(), "HashSet".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-010".into(),
                description: "Isogram checker".into(),
                prompt: "Write a class `IsogramChecker` with a method `isIsogram(String phrase)` that returns boolean. Use Set for detecting repeated letters.".into(),
                required_elements: vec!["class IsogramChecker".into(), "isIsogram".into(), "Set".into(), "boolean".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-011".into(),
                description: "Difference of squares".into(),
                prompt: "Write a class `DifferenceOfSquares` with methods `computeSquareOfSumTo(int n)`, `computeSumOfSquaresTo(int n)`, and `computeDifferenceOfSquares(int n)`.".into(),
                required_elements: vec!["class DifferenceOfSquares".into(), "computeSquareOfSumTo".into(), "computeSumOfSquaresTo".into(), "computeDifferenceOfSquares".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-012".into(),
                description: "Anagram".into(),
                prompt: "Write a class `Anagram` with a constructor taking the subject word and a method `match(List<String> candidates)` returning List<String> of valid anagrams.".into(),
                required_elements: vec!["class Anagram".into(), "List<String>".into(), "match".into(), "ArrayList".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-013".into(),
                description: "Clock".into(),
                prompt: "Write a class `Clock` with `Clock(int h, int m)`, `add(int minutes)`, `subtract(int minutes)` (returning new Clock), `equals(Object)`, `hashCode()`, and `toString()`.".into(),
                required_elements: vec!["class Clock".into(), "add".into(), "subtract".into(), "equals".into(), "hashCode".into(), "toString".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-014".into(),
                description: "Matrix".into(),
                prompt: "Write a class `Matrix` with `Matrix(String matrixAsString)` constructor and `getRow(int index)` and `getColumn(int index)` returning int[]. 1-indexed.".into(),
                required_elements: vec!["class Matrix".into(), "getRow".into(), "getColumn".into(), "int[]".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-015".into(),
                description: "Pascals triangle".into(),
                prompt: "Write a class `PascalsTriangle` with a static method `computeTriangle(int n)` that returns int[][] with the first n rows.".into(),
                required_elements: vec!["class PascalsTriangle".into(), "static".into(), "int[][]".into(), "computeTriangle".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-016".into(),
                description: "Sieve".into(),
                prompt: "Write a class `Sieve` with a static method `getPrimes(int limit)` returning List<Integer> of primes using the Sieve of Eratosthenes.".into(),
                required_elements: vec!["class Sieve".into(), "static".into(), "List<Integer>".into(), "getPrimes".into(), "boolean[]".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-017".into(),
                description: "Pythagorean triplet".into(),
                prompt: "Write a class `PythagoreanTriplet` with a static method `getTriplets(int sum)` that returns List of triplets (a,b,c) where a<b<c, a^2+b^2=c^2, and a+b+c=sum.".into(),
                required_elements: vec!["class PythagoreanTriplet".into(), "static".into(), "List".into(), "getTriplets".into(), "equals".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-018".into(),
                description: "Phone number".into(),
                prompt: "Write a class `PhoneNumber` with a constructor `PhoneNumber(String numberString)` and a method `getNumber()` that returns cleaned NANP number. Throw IllegalArgumentException for invalid inputs.".into(),
                required_elements: vec!["class PhoneNumber".into(), "PhoneNumber".into(), "getNumber".into(), "IllegalArgumentException".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-019".into(),
                description: "Grade school".into(),
                prompt: "Write a class `School` with methods `add(String student, int grade)`, `grade(int gradeLevel)` returning List<String>, and `roster()` returning Map<Integer, List<String>> sorted by grade and name.".into(),
                required_elements: vec!["class School".into(), "add".into(), "grade".into(), "roster".into(), "Map<Integer, List<String>>".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-020".into(),
                description: "Robot name".into(),
                prompt: "Write a class `Robot` with `getName()` returning String (two uppercase letters + three digits) and `reset()` generating a new name. Use Set for global uniqueness.".into(),
                required_elements: vec!["class Robot".into(), "getName".into(), "reset".into(), "Set<String>".into(), "Random".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-021".into(),
                description: "Word count".into(),
                prompt: "Write a class `WordCount` with a method `phrase(String input)` returning Map<String, Integer> with word occurrence counts.".into(),
                required_elements: vec!["class WordCount".into(), "Map<String, Integer>".into(), "HashMap".into(), "split".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-022".into(),
                description: "Luhn validator".into(),
                prompt: "Write a class `LuhnValidator` with a static method `isValid(String candidate)` that validates using the Luhn algorithm.".into(),
                required_elements: vec!["class LuhnValidator".into(), "static".into(), "boolean".into(), "isValid".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-023".into(),
                description: "Allergies".into(),
                prompt: "Write a class `Allergies` with `Allergies(int score)`, `isAllergicTo(Allergen allergen)` and `getList()` returning List<Allergen>. Use bitmask with an enum Allergen.".into(),
                required_elements: vec!["class Allergies".into(), "enum Allergen".into(), "isAllergicTo".into(), "getList".into(), "List<Allergen>".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-024".into(),
                description: "OCR numbers".into(),
                prompt: "Write a class `OCR` with a static method `convert(String input)` that converts 3x4 OCR grids to digits, returning '?' for unrecognizable.".into(),
                required_elements: vec!["class OCR".into(), "static".into(), "String".into(), "convert".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-025".into(),
                description: "Poker hands".into(),
                prompt: "Write a class `PokerHand` that implements Comparable and compares poker hands. Include rank calculation from high card through straight flush.".into(),
                required_elements: vec!["class PokerHand".into(), "Comparable".into(), "compareTo".into(), "enum".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-026".into(),
                description: "Bowling game".into(),
                prompt: "Write a class `BowlingGame` with `roll(int pins)` and `score()` methods. Track frames, strikes, spares, and 10th frame.".into(),
                required_elements: vec!["class BowlingGame".into(), "roll".into(), "score".into(), "int".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-027".into(),
                description: "Space age".into(),
                prompt: "Write a class `SpaceAge` with `SpaceAge(double seconds)` and methods for each planet returning age in years. Earth year = 31557600 seconds.".into(),
                required_elements: vec!["class SpaceAge".into(), "double".into(), "seconds".into(), "getAge".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-028".into(),
                description: "Roman numerals".into(),
                prompt: "Write a class `RomanNumeral` with a static method `toRoman(int number)` that converts 1-3999 to Roman numerals.".into(),
                required_elements: vec!["class RomanNumeral".into(), "static".into(), "String".into(), "toRoman".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-029".into(),
                description: "Proverb".into(),
                prompt: "Write a class `Proverb` with a static method `proverb(String[] words)` returning String with the full proverb.".into(),
                required_elements: vec!["class Proverb".into(), "static".into(), "String".into(), "proverb".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-030".into(),
                description: "Binary search tree".into(),
                prompt: "Write a generic class `BinarySearchTree<T extends Comparable<T>>` with `insert(T value)`, and `getSortedList()` returning List<T>.".into(),
                required_elements: vec!["class BinarySearchTree".into(), "extends Comparable".into(), "insert".into(), "getSortedList".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-031".into(),
                description: "Simple cipher".into(),
                prompt: "Write a class `Cipher` with `Cipher(String key)`, `encode(String plain)`, and `decode(String cipher)` methods implementing a substitution cipher. Throw for invalid key.".into(),
                required_elements: vec!["class Cipher".into(), "String".into(), "encode".into(), "decode".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-032".into(),
                description: "Series".into(),
                prompt: "Write a class `Series` with a constructor taking a string and a method `slices(int n)` returning List<String> of all consecutive substrings of length n.".into(),
                required_elements: vec!["class Series".into(), "List<String>".into(), "slices".into(), "ArrayList".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-033".into(),
                description: "Diamond pattern".into(),
                prompt: "Write a class `DiamondPrinter` with a method `printToList(char letter)` returning List<String> with diamond pattern starting from 'A'.".into(),
                required_elements: vec!["class DiamondPrinter".into(), "List<String>".into(), "printToList".into(), "char".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-034".into(),
                description: "List flattening".into(),
                prompt: "Write a class `Flattener` with a static method `flatten(List<Object> list)` that flattens nested lists, filtering null values. Use recursion.".into(),
                required_elements: vec!["class Flattener".into(), "static".into(), "List<Object>".into(), "flatten".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-035".into(),
                description: "Minesweeper".into(),
                prompt: "Write a class `MinesweeperBoard` with `MinesweeperBoard(List<String> board)` and `withNumbers()` returning List<String> with adjacent mine counts.".into(),
                required_elements: vec!["class MinesweeperBoard".into(), "List<String>".into(), "withNumbers".into(), "charAt".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-036".into(),
                description: "Circular buffer".into(),
                prompt: "Write a generic class `CircularBuffer<T>` with `CircularBuffer(int capacity)`, `read()`, `write(T value)`, `overwrite(T value)`, `clear()` methods. Throw BufferEmptyException/BufferFullException.".into(),
                required_elements: vec!["class CircularBuffer".into(), "<T>".into(), "read".into(), "write".into(), "overwrite".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-037".into(),
                description: "Markdown parser".into(),
                prompt: "Write a class `MarkdownParser` with a static method `parse(String markdown)` that converts simple Markdown (headings #, **bold**, *italic*, paragraphs, lists) to HTML.".into(),
                required_elements: vec!["class MarkdownParser".into(), "static".into(), "String".into(), "parse".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
            BenchmarkTask {
                id: "JV-038".into(),
                description: "Saddle points".into(),
                prompt: "Write a class `Matrix` with `Matrix(List<List<Integer>> values)`, `getSaddlePoints()` returning Set<MatrixCoordinate> where MatrixCoordinate is a record with row and column.".into(),
                required_elements: vec!["class Matrix".into(), "Set<MatrixCoordinate>".into(), "getSaddlePoints".into(), "record MatrixCoordinate".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("java".into()),
            },
        ]);

        // =====================================================================
        // C++ (38 tasks: CP-001 to CP-038)
        // =====================================================================
        all.extend(vec![
            BenchmarkTask {
                id: "CP-001".into(),
                description: "Hello World".into(),
                prompt: "Write a function `std::string hello(const std::string& name)` that returns a greeting. Use 'World' if name is empty.".into(),
                required_elements: vec!["std::string".into(), "hello".into(), "return".into(), "#include".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-002".into(),
                description: "Leap year".into(),
                prompt: "Write a function `bool is_leap_year(int year)` that returns true if the year is a leap year.".into(),
                required_elements: vec!["bool".into(), "is_leap_year".into(), "return".into(), "int".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-003".into(),
                description: "Reverse string".into(),
                prompt: "Write a function `std::string reverse_string(const std::string& s)` that returns the reversed string.".into(),
                required_elements: vec!["std::string".into(), "reverse_string".into(), "rbegin".into(), "rend".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-004".into(),
                description: "RNA transcription".into(),
                prompt: "Write a function `std::string to_rna(const std::string& dna)` that returns RNA complement using a switch statement or map.".into(),
                required_elements: vec!["std::string".into(), "to_rna".into(), "switch".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-005".into(),
                description: "Raindrops".into(),
                prompt: "Write a function `std::string raindrops(int n)` that returns Pling/Plang/Plong string using std::string append.".into(),
                required_elements: vec!["std::string".into(), "raindrops".into(), "+= ".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-006".into(),
                description: "Scrabble score".into(),
                prompt: "Write a function `int score(const std::string& word)` that returns Scrabble score using std::map or std::unordered_map.".into(),
                required_elements: vec!["int".into(), "score".into(), "std::map".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-007".into(),
                description: "Gigasecond".into(),
                prompt: "Write a function `std::chrono::system_clock::time_point gigasecond_after(std::chrono::system_clock::time_point start)` that adds 1e9 seconds.".into(),
                required_elements: vec!["std::chrono".into(), "gigasecond_after".into(), "system_clock".into(), "time_point".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-008".into(),
                description: "Hamming distance".into(),
                prompt: "Write a function `int hamming_distance(const std::string& a, const std::string& b)` that returns Hamming distance. Throw std::invalid_argument if lengths differ.".into(),
                required_elements: vec!["int".into(), "hamming_distance".into(), "std::invalid_argument".into(), "throw".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-009".into(),
                description: "Acronym".into(),
                prompt: "Write a function `std::string acronym(const std::string& phrase)` that returns the acronym, handling hyphens and spaces.".into(),
                required_elements: vec!["std::string".into(), "acronym".into(), "std::isalpha".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-010".into(),
                description: "Collatz conjecture".into(),
                prompt: "Write a function `int collatz_conjecture(int n)` that returns the number of steps to reach 1. Throw std::invalid_argument if n <= 0.".into(),
                required_elements: vec!["int".into(), "collatz_conjecture".into(), "throw".into(), "std::invalid_argument".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-011".into(),
                description: "Pangram".into(),
                prompt: "Write a function `bool is_pangram(const std::string& sentence)` that checks for all 26 letters. Case-insensitive. Use std::set or bitset.".into(),
                required_elements: vec!["bool".into(), "is_pangram".into(), "std::set".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-012".into(),
                description: "Isogram".into(),
                prompt: "Write a function `bool is_isogram(const std::string& word)` that checks for repeated letters. Use std::set or bool array.".into(),
                required_elements: vec!["bool".into(), "is_isogram".into(), "std::set".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-013".into(),
                description: "Difference of squares".into(),
                prompt: "Write functions `int square_of_sum(int n)`, `int sum_of_squares(int n)`, and `int difference(int n)`.".into(),
                required_elements: vec!["int".into(), "square_of_sum".into(), "sum_of_squares".into(), "difference".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-014".into(),
                description: "Anagram".into(),
                prompt: "Write a function `std::vector<std::string> anagram_matches(const std::string& subject, const std::vector<std::string>& candidates)` that returns matching anagrams.".into(),
                required_elements: vec!["std::vector<std::string>".into(), "anagram_matches".into(), "std::sort".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-015".into(),
                description: "Clock class".into(),
                prompt: "Write a `Clock` class with `Clock(int h, int m)`, 24-hour wrap, overloaded `operator+` (int minutes), `operator-` (int minutes), `operator==`, and `operator<<` for output.".into(),
                required_elements: vec!["class Clock".into(), "operator+".into(), "operator-".into(), "operator==".into(), "friend".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-016".into(),
                description: "Sieve".into(),
                prompt: "Write a function `std::vector<int> sieve(int limit)` that returns primes up to limit using std::vector<bool> as boolean array.".into(),
                required_elements: vec!["std::vector<int>".into(), "sieve".into(), "std::vector<bool>".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-017".into(),
                description: "Pascals triangle".into(),
                prompt: "Write a function `std::vector<std::vector<int>> pascals_triangle(int n)` returning the first n rows.".into(),
                required_elements: vec!["std::vector<std::vector<int>>".into(), "pascals_triangle".into(), "push_back".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-018".into(),
                description: "Luhn validator".into(),
                prompt: "Write a function `bool luhn_valid(const std::string& number)` that validates using the Luhn algorithm.".into(),
                required_elements: vec!["bool".into(), "luhn_valid".into(), "return".into(), "std::isdigit".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-019".into(),
                description: "Robot name".into(),
                prompt: "Write a `Robot` class with `std::string name() const` and `void reset()`. Generate names like 'AB123' with uniqueness via static std::set.".into(),
                required_elements: vec!["class Robot".into(), "std::string".into(), "name".into(), "reset".into(), "static".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-020".into(),
                description: "Allergies".into(),
                prompt: "Write an `Allergies` class with `Allergies(int score)`, `bool is_allergic_to(const std::string& substance)` and `std::vector<std::string> get_allergies()`. Use bitmask and enum.".into(),
                required_elements: vec!["class Allergies".into(), "enum".into(), "is_allergic_to".into(), "get_allergies".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-021".into(),
                description: "Word count".into(),
                prompt: "Write a function `std::map<std::string, int> word_count(const std::string& phrase)` that counts word occurrences, handling contractions.".into(),
                required_elements: vec!["std::map<std::string, int>".into(), "word_count".into(), "return".into(), "std::stringstream".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-022".into(),
                description: "Binary search".into(),
                prompt: "Write a function `int binary_search(const std::vector<int>& arr, int target)` that returns index or -1.".into(),
                required_elements: vec!["int".into(), "binary_search".into(), "std::vector<int>".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-023".into(),
                description: "Perfect numbers".into(),
                prompt: "Write a function `std::string classify(int n)` that returns 'perfect', 'abundant', or 'deficient' based on sum of proper divisors.".into(),
                required_elements: vec!["std::string".into(), "classify".into(), "return".into(), "perfect".into(), "abundant".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-024".into(),
                description: "Matrix class".into(),
                prompt: "Write a `Matrix` class with `Matrix(const std::string& input)`, `std::vector<int> row(int n)`, `std::vector<int> column(int n)`. Parse space and newline separated ints.".into(),
                required_elements: vec!["class Matrix".into(), "std::vector<int>".into(), "row".into(), "column".into(), "std::stringstream".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-025".into(),
                description: "Prime factors".into(),
                prompt: "Write a function `std::vector<int> prime_factors(int n)` that returns prime factors in ascending order.".into(),
                required_elements: vec!["std::vector<int>".into(), "prime_factors".into(), "while".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-026".into(),
                description: "Phone number".into(),
                prompt: "Write a `PhoneNumber` class for NANP format with constructor `PhoneNumber(const std::string& n)` and `std::string number()` method. Throw std::domain_error on invalid input.".into(),
                required_elements: vec!["class PhoneNumber".into(), "std::string".into(), "number".into(), "std::domain_error".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-027".into(),
                description: "Grade school".into(),
                prompt: "Write a `GradeSchool` class with `void add(const std::string& name, int grade)`, `std::vector<std::string> grade(int n)`, and `std::map<int, std::vector<std::string>> roster()`.".into(),
                required_elements: vec!["class GradeSchool".into(), "add".into(), "grade".into(), "roster".into(), "std::map".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-028".into(),
                description: "Nth prime".into(),
                prompt: "Write a function `int nth_prime(int n)` that returns the nth prime number. Throw std::invalid_argument for n <= 0.".into(),
                required_elements: vec!["int".into(), "nth_prime".into(), "throw".into(), "std::invalid_argument".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-029".into(),
                description: "Roman numerals".into(),
                prompt: "Write a function `std::string to_roman(int n)` that converts 1-3999 to Roman numerals using subtractive notation.".into(),
                required_elements: vec!["std::string".into(), "to_roman".into(), "return".into(), "constexpr".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-030".into(),
                description: "Space age".into(),
                prompt: "Write a `SpaceAge` class with `SpaceAge(long long seconds)` and methods `on_earth()`, `on_mercury()`, etc. Use constexpr orbital period ratios.".into(),
                required_elements: vec!["class SpaceAge".into(), "long long".into(), "on_earth".into(), "constexpr".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-031".into(),
                description: "Pythagorean triplet".into(),
                prompt: "Write a function `std::vector<std::tuple<int,int,int>> pythagorean_triplets(int sum)` returning all triplets a<b<c with a^2+b^2=c^2 and a+b+c=sum.".into(),
                required_elements: vec!["std::vector<std::tuple<int,int,int>>".into(), "pythagorean_triplets".into(), "for".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-032".into(),
                description: "Simple linked list".into(),
                prompt: "Write a template `singly_linked_list<T>` class with `push_front(T)`, `pop_front()`, `empty()`, `size()` methods. Use unique_ptr or raw new/delete for nodes.".into(),
                required_elements: vec!["template".into(), "class".into(), "push_front".into(), "pop_front".into(), "struct Node".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-033".into(),
                description: "Beer song".into(),
                prompt: "Write a function `std::string beer_song(int start, int end)` returning verses of '99 Bottles of Beer' with correct pluralization.".into(),
                required_elements: vec!["std::string".into(), "beer_song".into(), "std::to_string".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-034".into(),
                description: "Minesweeper board".into(),
                prompt: "Write a function `std::vector<std::string> annotate(const std::vector<std::string>& board)` that adds mine counts ('*' = mine). Use nested for loops with boundary checking.".into(),
                required_elements: vec!["std::vector<std::string>".into(), "annotate".into(), "size".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-035".into(),
                description: "Circular buffer".into(),
                prompt: "Write a template `circular_buffer<T>` class with `circular_buffer(size_t capacity)`, `read(T&)`, `write(const T&)`, `overwrite(const T&)`, `clear()`. Use std::vector as underlying storage.".into(),
                required_elements: vec!["template".into(), "class circular_buffer".into(), "read".into(), "write".into(), "clear".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-036".into(),
                description: "Bob".into(),
                prompt: "Write a function `std::string bob_response(const std::string& message)` that returns Bob's responses based on message analysis (question, yell, silence).".into(),
                required_elements: vec!["std::string".into(), "bob_response".into(), "return".into(), "std::isupper".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-037".into(),
                description: "Sieve (modern C++)".into(),
                prompt: "Write a function `std::vector<int> prime_sieve(int limit)` using modern C++ (range-based for, std::vector<bool>, auto).".into(),
                required_elements: vec!["std::vector<int>".into(), "prime_sieve".into(), "auto".into(), "range".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
            BenchmarkTask {
                id: "CP-038".into(),
                description: "Saddle points".into(),
                prompt: "Write a function `std::set<std::pair<int,int>> saddle_points(const std::vector<std::vector<int>>& matrix)` that finds all saddle points.".into(),
                required_elements: vec!["std::set<std::pair<int,int>>".into(), "saddle_points".into(), "for".into(), "return".into()],
                forbidden_elements: vec![],
                validation_type: "code".into(), known_answer: None, tolerance: None, reference: None, language: Some("cpp".into()),
            },
        ]);

        assert_eq!(all.len(), 228, "Expected 228 tasks (38 per language x 6 languages)");
        all
    }
}

impl BenchmarkRunner for AiderPolyglotRunner {
    fn id(&self) -> &str {
        "aider-polyglot"
    }

    fn metadata(&self) -> BenchmarkMetadata {
        BenchmarkMetadata {
            id: "aider-polyglot".to_string(),
            name: "Aider Polyglot — Multi-Language Code Editing".to_string(),
            description: "228 Exercism-based exercises across 6 languages (C++, Go, Java, JavaScript, Python, Rust). Measures multi-language code generation, language-specific API knowledge, and cross-language engineering skill.".to_string(),
            task_count: Self::tasks().len(),
            max_level: 3,
            languages: vec![
                "cpp".to_string(),
                "go".to_string(),
                "java".to_string(),
                "javascript".to_string(),
                "python".to_string(),
                "rust".to_string(),
            ],
            requires_docker: false,
            requires_network: false,
            estimated_duration_minutes: 90,
            tier: "tier-1".to_string(),
        }
    }

    fn discover_tasks(&self) -> Result<Vec<BenchmarkTask>, String> {
        Ok(Self::tasks())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let runner = AiderPolyglotRunner::new();
        let meta = runner.metadata();
        assert_eq!(meta.id, "aider-polyglot");
        assert_eq!(meta.task_count, 228);
        assert_eq!(meta.languages.len(), 6);
        assert!(meta.languages.contains(&"rust".to_string()));
        assert!(meta.languages.contains(&"python".to_string()));
        assert!(meta.languages.contains(&"go".to_string()));
        assert!(meta.languages.contains(&"javascript".to_string()));
        assert!(meta.languages.contains(&"java".to_string()));
        assert!(meta.languages.contains(&"cpp".to_string()));
    }

    #[test]
    fn test_discover_tasks_count() {
        let runner = AiderPolyglotRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        assert_eq!(tasks.len(), 228, "Must have exactly 228 tasks");
    }

    #[test]
    fn test_tasks_organized_by_language() {
        let tasks = AiderPolyglotRunner::tasks();
        // Check that all language prefixes exist
        let prefixes: std::collections::HashSet<&str> = tasks.iter().map(|t| &t.id[..2]).collect();
        assert!(prefixes.contains("PY"), "Missing Python tasks (PY-*)");
        assert!(prefixes.contains("GO"), "Missing Go tasks (GO-*)");
        assert!(prefixes.contains("RS"), "Missing Rust tasks (RS-*)");
        assert!(prefixes.contains("JS"), "Missing JavaScript tasks (JS-*)");
        assert!(prefixes.contains("JV"), "Missing Java tasks (JV-*)");
        assert!(prefixes.contains("CP"), "Missing C++ tasks (CP-*)");
        assert_eq!(prefixes.len(), 6, "Should have exactly 6 language prefixes");
    }

    #[test]
    fn test_each_language_has_38_tasks() {
        let tasks = AiderPolyglotRunner::tasks();
        for prefix in &["PY", "GO", "RS", "JS", "JV", "CP"] {
            let count = tasks.iter().filter(|t| t.id.starts_with(prefix)).count();
            assert_eq!(count, 38, "Language {prefix} should have exactly 38 tasks, got {count}");
        }
    }

    #[test]
    fn test_validate_python_task() {
        let runner = AiderPolyglotRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let py_task = tasks.iter().find(|t| t.id == "PY-001").unwrap();

        // Valid Python Hello World
        let valid = "def hello(name):\n    if not name:\n        return 'Hello, World!'\n    return f'Hello, {name}!'";
        let v_result = resolve_validator("code").validate(py_task, valid).unwrap(); assert!(v_result.passed);
    }

    #[test]
    fn test_validate_rust_task() {
        let runner = AiderPolyglotRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let rs_task = tasks.iter().find(|t| t.id == "RS-001").unwrap();

        // Valid Rust Hello World
        let valid = "pub fn hello(name: Option<&str>) -> String {\n    match name {\n        Some(n) if !n.is_empty() => format!(\"Hello, {n}!\"),\n        _ => \"Hello, World!\".to_string(),\n    }\n}";
        let v_result = resolve_validator("code").validate(rs_task, valid).unwrap(); assert!(v_result.passed);
    }

    #[test]
    fn test_validate_go_task() {
        let runner = AiderPolyglotRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let go_task = tasks.iter().find(|t| t.id == "GO-004").unwrap();

        // Valid Go Raindrops — check it passes required elements
        let valid = "package raindrops\n\nfunc Convert(number int) string {\n    var result string\n    if number%3 == 0 { result += \"Pling\" }\n    if number%5 == 0 { result += \"Plang\" }\n    if number%7 == 0 { result += \"Plong\" }\n    if result == \"\" { result = fmt.Sprintf(\"%d\", number) }\n    return result\n}";
        let v_result = resolve_validator("code").validate(go_task, valid).unwrap(); assert!(v_result.passed);
    }

    #[test]
    fn test_validate_fails_missing_required() {
        let runner = AiderPolyglotRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let task = tasks.iter().find(|t| t.id == "CP-001").unwrap();

        // Missing std::string
        let bad = "void hello() {}";
        let v_result = resolve_validator("code").validate(task, bad).unwrap(); assert!(!v_result.passed);
    }

    #[test]
    fn test_validate_fails_forbidden() {
        let runner = AiderPolyglotRunner::new();
        let tasks = runner.discover_tasks().unwrap();
        let rs_task = tasks.iter().find(|t| t.id == "RS-034").unwrap();

        // RS-034 forbids .map()
        let bad = "fn map_function(values: &[T], f: fn(&T) -> U) -> Vec<U> {\n    values.iter().map(f).collect()\n}";
        let v_result = resolve_validator("code").validate(rs_task, bad).unwrap(); assert!(!v_result.passed);
    }

    #[test]
    fn test_language_hints_set() {
        let tasks = AiderPolyglotRunner::tasks();
        for task in &tasks {
            assert!(
                task.language.is_some(),
                "Task {} has no language hint",
                task.id
            );
        }
    }
}
