# Basic Latin → Cyrillic Mapping Table
# Russian Phonetic Transliteration

a   -> а
b   -> б
c   -> к
d   -> д
e   -> е
f   -> ф
g   -> г
h   -> х
i   -> и
j   -> дж
k   -> к
l   -> л
m   -> м
n   -> н
o   -> о
p   -> п
q   -> к
r   -> р
s   -> с
t   -> т
u   -> у
v   -> в
w   -> в
x   -> кс
y   -> й
z   -> з


# Uppercase

A   -> А
B   -> Б
C   -> К
D   -> Д
E   -> Е
F   -> Ф
G   -> Г
H   -> Х
I   -> И
J   -> Дж
K   -> К
L   -> Л
M   -> М
N   -> Н
O   -> О
P   -> П
Q   -> К
R   -> Р
S   -> С
T   -> Т
U   -> У
V   -> В
W   -> В
X   -> Кс
Y   -> Й
Z   -> З


# Multi-character Priority Rules
# IMPORTANT:
# Longest pattern MUST be matched first

sch     -> щ
Sch     -> Щ
SCH     -> Щ

shch    -> щ
Shch    -> Щ
SHCH    -> Щ

yo      -> ё
Yo      -> Ё
YO      -> Ё

zh      -> ж
Zh      -> Ж
ZH      -> Ж

kh      -> х
Kh      -> Х
KH      -> Х

ts      -> ц
Ts      -> Ц
TS      -> Ц

ch      -> ч
Ch      -> Ч
CH      -> Ч

sh      -> ш
Sh      -> Ш
SH      -> Ш

yu      -> ю
Yu      -> Ю
YU      -> Ю

ya      -> я
Ya      -> Я
YA      -> Я

ye      -> е
Ye      -> Е
YE      -> Е

yi      -> ы
Yi      -> Ы
YI      -> Ы

eh      -> э
Eh      -> Э
EH      -> Э

ju      -> ю
Ju      -> Ю
JU      -> Ю

ja      -> я
Ja      -> Я
JA      -> Я


# Soft / Hard Signs

'       -> ь
''      -> ъ


# Special Cases

iy      -> ий
oy      -> ой
ey      -> ей

ks      -> кс
Ks      -> Кс

yae     -> яе
yoi     -> ёи


# Numbers and Symbols
# Pass-through (unchanged)

0 -> 0
1 -> 1
2 -> 2
3 -> 3
4 -> 4
5 -> 5
6 -> 6
7 -> 7
8 -> 8
9 -> 9

. -> .
, -> ,
! -> !
? -> ?
: -> :
; -> ;
( -> (
) -> )
[ -> [
] -> ]
{ -> {
} -> }
+ -> +
- -> -
* -> *
/ -> /
= -> =
_ -> _
@ -> @
# -> #
$ -> $
% -> %
^ -> ^
& -> &
