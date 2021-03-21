# payments-engine

BASICS: Codebase builds, reads and writes data as specified. I also included a function that looks 
for preexisting accounts data. However, unless we change the current commandline to run the engine
that data gets overwritten. However, this is an easy fix, and I left it in there as it made more 
sense to me that in a real world scenario we would be reading in preexisting accounts data.

COMPLETENESS: all cases were covered. However, one case I discovered through unit testing
is not properly covered. If a withdrawal exceeds the available amount nothing should happen.
This is implemented. But what is not taken into account is that such a transaction could 
in the current implementation in theory be disputed. This does not make much sense to me as
the transaction was not actually processed. It was not clear to me if this case should be 
coded as well or not so I left it for now.

CORRECTNESS: I tested against handmade transactions. Found the above "dispute case" that needs 
clarification for me. Apart from this rudimentary testing I'm heavily relying on Rust type system 
to ensure correctness. Specifically the transaction processing logic is split neatly out into small 
easy-to-read functions.

SAFETY AND ROBUSTNESS: No unsafe code. Handled all possible errors and handled invalid transaction data. 
Only two unwraps in code where it's clear that no panic will result.

EFFICIENCY: Given the large amounts of expected data I do not read the entire transactions.csv but instead
makes used of buffered reading. I read in a row, create a Transaction instance, and then automatically free 
the memory of that row string.

MAINTAINABILITY: All code is in one script now. Would be nicer to clean this up into 2-3 modules. At some places 
refactoring some structs could lead to shorter and more transparent code. I've made use of Rust's automatic 
documentation functionality. If we run cargo doc we'll find the documentation for all the code base in the doc folder.

