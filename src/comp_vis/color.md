Let $r,g,b \in \{0, \dots, 255\}$

Let $n \in \N$. By the closure of natural numbers under multiplication, $n^2 \in \N$. And more specifically, $n^2 \in \{0^2, 1^2, 2^2, 3^2, 4^2, \dots\}$. Therefore, $\sqrt{n^2} \in \N$, meaning $\lfloor\sqrt{n^2}\rfloor = \sqrt{n^2}$.

Let $a,b \in \N$. Then $a^2,b^2 \in \{0^2, 1^2, 2^2, 3^2, 4^2, \dots\}$. Now, what of $a^2+b^2$?

$$
\begin{align*}
    a^2+b^2
        &\in \{0^2, 1^2, 2^2, 3^2, 4^2, \dots\} + \{0^2, 1^2, 2^2, 3^2, 4^2, \dots\} \\
        &\in \{0, 1, 4, 9, 16, \dots\} + \{0, 1, 4, 9, 16, \dots\} \\
        &\in \begin{Bmatrix}
             0 &  1 &  4 &  9 & 16 & \dots \\ % 0+n
             1 &  2 &  5 & 10 & 17 & \dots \\ % 1+n
             4 &  5 &  8 & 13 & 20 & \dots \\ % 4+n
             9 & 10 & 13 & 18 & 25 & \dots \\ % 9+n
            16 & 17 & 20 & 25 & 32 & \dots \\ % 16+n
            & & \vdots
        \end{Bmatrix} \\ % this reminds me of pascal's triangle...
        &\in \begin{Bmatrix}
             0 &  1 &  4 &  9 & 16 & 25 & 36 & \dots \\ % 0+n
               &  2 &  5 & 10 & 17 & 26 & 37 & \dots \\ % 1+n
               &    &  8 & 13 & 20 & 29 & 40 & \dots \\ % 4+n
               &    &    & 18 & 25 & 34 & 45 & \dots \\ % 9+n
               &    &    &    & 32 & 41 & 52 & \dots \\ % 16+n
               &    &    &    &    & 50 & 61 & \dots \\ % 25+n
               &    &    &    &    &    & 72 & \dots \\ % 36+n
            & & & & & & & \ddots
        \end{Bmatrix} \\
        &5+8=13 \\
        &2\cdot10=20 \\
        &\text{row 2 col 2 = 2 which everything on the diagonal is multiplied by...} \\
        &\text{let the first row be \(n\) and the column be \(i\)} \\
        &\in \begin{Bmatrix}
             n & n  & n         & n         & n         & n         & n         & \dots \\
               & 2n & n+n_{i-1} & n+2^0     & n+2^0     & n+2^0     & n+2^0     & \dots \\
               &    & 2n        & n+n_{i-1} & n+2^2     & n+2^2     & n+2^2     & \dots \\
               &    &           & 2n        & n+n_{i-1} & n+ 9      & n+9       & \dots \\
               &    &           &           & 2n        & n+n_{i-1} & n+2^3     & \dots \\
               &    &           &           &           & 2n        & n+n_{i-1} & \dots \\
               &    &           &           &           &           & 2n        & \dots \\
            & & & & & & & \ddots
        \end{Bmatrix} \\
        &\text{let the previous row be \(n\)} \\
        &\in \begin{Bmatrix}
             n & n   & n   & n   & n   & n   & n    & \dots \\
               & n+1 & n+1 & n+1 & n+1 & n+1 & n+1  & \dots \\
               &     & n+3 & n+3 & n+3 & n+3 & n+3  & \dots \\
               &     &     & n+5 & n+5 & n+5 & n+5  & \dots \\
               &     &     &     & n+7 & n+7 & n+7  & \dots \\
               &     &     &     &     & n+9 & n+9  & \dots \\
               &     &     &     &     &     & n+11 & \dots \\
            & & & & & & & \ddots
        \end{Bmatrix} \\
        &\text{let the previous row be \(n\) and the current row number be \(r\) starting from 0} \\
        &\in \begin{Bmatrix}
             n & n   & n   & n   & n   & n   & n    & \dots \\
               & n+2r-1 & n+2r-1 & n+2r-1 & n+2r-1 & n+2r-1 & n+2r-1  & \dots \\
               &     & n+2r-1 & n+2r-1 & n+2r-1 & n+2r-1 & n+2r-1  & \dots \\
               &     &     & n+2r-1 & n+2r-1 & n+2r-1 & n+2r-1  & \dots \\
               &     &     &     & n+2r-1 & n+2r-1 & n+2r-1  & \dots \\
               &     &     &     &     & n+2r-1 & n+2r-1  & \dots \\
               &     &     &     &     &     & n+2r-1 & \dots \\
            & & & & & & & \ddots
        \end{Bmatrix} \\
        &\text{let the column be \(c\) starting from 0} \\
        &\in \begin{Bmatrix}
             c^2 & c^2   & c^2   & c^2   & c^2   & c^2   & c^2    & \dots \\
               & c^2+2r-1 & c^2+2r-1 & c^2+2r-1 & c^2+2r-1 & c^2+2r-1 & c^2+2r-1  & \dots \\
               &     & c^2+2(2r-1) & c^2+2(2r-1) & c^2+2(2r-1) & c^2+2(2r-1) & c^2+2(2r-1)  & \dots \\
               &     &     & c^2+2r-1+2r-1+2r-1 & c^2+2r-1+2r-1+2r-1 & c^2+2r-1+2r-1+2r-1 & c^2+2r-1+2r-1+2r-1  & \dots \\
               &     &     &     & n+2r-1 & n+2r-1 & n+2r-1  & \dots \\
               &     &     &     &     & n+2r-1 & n+2r-1  & \dots \\
               &     &     &     &     &     & n+2r-1 & \dots \\
            & & & & & & & \ddots
        \end{Bmatrix} \\
\end{align*}
$$

We want to calculate $\sqrt{a^2+b^2}$ without $\sqrt{}$ if possible, given $a,b \in \N$.

$$
\begin{gather*}
    \left\{\begin{array}{c|cccccccc}
        + & 0^2 & 1^2 & 2^2 & 3^2 & 4^2 & 5^2 & 6^2 & \dots \\
        \hline
        0^2 & 0 &  1 &  4 &  9 & 16 & 25 & 36 & \dots \\ % 0+n
        1^2 & &  2 &  5 & 10 & 17 & 26 & 37 & \dots \\ % 1+n
        2^2 & &    &  8 & 13 & 20 & 29 & 40 & \dots \\ % 4+n
        3^2 & &    &    & 18 & 25 & 34 & 45 & \dots \\ % 9+n
        4^2 & &    &    &    & 32 & 41 & 52 & \dots \\ % 16+n
        5^2 & &    &    &    &    & 50 & 61 & \dots \\ % 25+n
        6^2 & &    &    &    &    &    & 72 & \dots \\ % 36+n
        7^2 & & & & & & & & \ddots \\
        \vdots
    \end{array}\right\} \\
    \left\{\begin{array}{c|cccccccc}
        \sqrt{\sum} & 0^2 & 1^2 & 2^2 & 3^2 & 4^2 & 5^2 & 6^2 & \dots \\
        \hline
        0^2 & 0 &  1 &  2 &  3 & 4 & 5 & 6 & \dots \\ % 0+n
        1^2 & &  \sqrt{2} &  \sqrt{5} & \sqrt{10} & \sqrt{17} & \sqrt{26} & \sqrt{37} & \dots \\ % 1+n
        2^2 & &    &  \sqrt{8} & \sqrt{13} & \sqrt{20} & \sqrt{29} & \sqrt{40} & \dots \\ % 4+n
        3^2 & &    &    & \sqrt{18} & 5 & \sqrt{34} & \sqrt{45} & \dots \\ % 9+n
        4^2 & &    &    &    & \sqrt{32} & \sqrt{41} & \sqrt{52} & \dots \\ % 16+n
        5^2 & &    &    &    &    & \sqrt{50} & \sqrt{61} & \dots \\ % 25+n
        6^2 & &    &    &    &    &    & \sqrt{72} & \dots \\ % 36+n
        7^2 & & & & & & & & \ddots \\
        \vdots
    \end{array}\right\}
\end{gather*}
$$

darn...

what about the dot product of two normalized vectors?

Note: "$\bull$" here means dot product, NOT multiplication
$$
\begin{align*}
    \hat{u} \bull \hat{v}
        &= \frac{u}{\lvert u \rvert} \bull \frac{v}{\lvert v \rvert} & \text{definition of unit vector} \\
        &= \left(\frac{u_x}{\lvert u \rvert}\right)\left(\frac{v_x}{\lvert v \rvert}\right) + \left(\frac{u_y}{\lvert u \rvert}\right)\left(\frac{v_y}{\lvert v \rvert}\right) + \left(\frac{u_z}{\lvert u \rvert}\right)\left(\frac{v_z}{\lvert v \rvert}\right) & \text{definition of dot product} \\
        &= \frac{u_xv_x}{\lvert u \rvert\lvert v \rvert} + \frac{u_yv_y}{\lvert u \rvert\lvert v \rvert} + \frac{u_zv_z}{\lvert u \rvert\lvert v \rvert} &\text{multiply fractions} \\
        &= \frac{u_xv_x+u_yv_y+u_zv_z}{\lvert u \rvert\lvert v \rvert} &\text{shared denominator} \\
        &= \frac{u \bull v}{\lvert u \rvert\lvert v \rvert} &\text{definition of dot product} \\
        &= \frac{u \bull v}{\sqrt{\lVert u \rVert}\sqrt{\lVert v \rVert}} \\
        &= \frac{u \bull v}{\sqrt{\lVert u \rVert\lVert v \rVert}} \\
\end{align*}
$$

# Properties of Dot Product

Let $u,v,w$ be vectors and $k$ be a scalar

## Commutative
$$u \bull v = v \bull u$$

## Distributive
$$u \bull (v + w) = u \bull v + u \bull w$$

## Scalar associative
$$k(u \bull v) = ku \bull v = u \bull kv$$

## Zero
$$0 \bull u = 0$$

## Norm
$$u \bull u = \lVert u \rVert$$
