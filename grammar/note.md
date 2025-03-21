

```
S -> if E then M1 S1 N else M2 S2
```
- N: 执行完S1后跳过执行S2
- M1: E为假时跳过S1

```
for ( A B C ) { D }
```
转换为
```
A
while B { D C }
```