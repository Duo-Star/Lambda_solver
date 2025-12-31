pub static HELLO_TEXT: &str = r"--/ Lambda Slover 1.0.0
2025 - 12 - 31 - Duo
MathForest 欢迎你的加入 - 663251235
/
----------------------预载宏------------------------
[0] : (\fx.x);
[1] : (\fx.fx);
[2] : (\fx.f(fx));
[3] : (\fx.f(f(fx)));
[4] : (\fx.f(f(f(fx))));
[5] : (\fx.f(f(f(f(fx)))));
[6] : (\fx.f(f(f(f(f(fx))))));
[7] : (\fx.f(f(f(f(f(f(fx)))))));
[8] : (\fx.f(f(f(f(f(f(f(fx))))))));
[9] : (\fx.f(f(f(f(f(f(f(f(fx)))))))));
-------------------------------------------------
[T]   : (\xy.x);
[F]   : (\xy.y);
[If]  : (\pab.pab);
[And] : (\pq.pqp);
[Or]  : (\pq.ppq);
[Not] : ((\p.p(\ab.b)(\ab.a)));
-------------------------------------------------
[+1] : (\nfx.f(nfx));
[-1] : (\nfx.n(\gh.h(gf))(\u.x)(\u.u));
[+]  : (\mnfx.mf(nfx));
[-]  : (\mn.n(\kfx.k(\gh.h(gf))(\u.x)(\u.u))m);
[*]  : (\mnfx.m(nf)x);
[/]  : \nm. [If] ([<] n m) [0] ([+1] ([Div] ([-] n m) m));
[^]  : (\mn.nm);
[?0] : (\n.n(\x.\ab.b)(\ab.a));
[<=] : (\mn.(\n.n(\x.\ab.b)(\ab.a)) ((\mn.n(\kfx.k(\gh.h(gf))(\u.x)(\u.u))m) m n));
[<]  : \mn. [Not] ([<=] n m);
[Mod]: \mn. [If] ([<] m n) m ([Mod] ([-] m n) n);
[?Div] : \dn. [?0] ([Mod] n d);
------------------Lambda here!-------------------
";

pub static DEMOS: [[&str; 2]; 5] = [
    //
    ["1+1", r"[+] [1] [1]"],
    //
    ["bool", r"[Or] ([Not] [T]) [T]"],
    //
    ["素数", r"[Check]   : \nd. [If] ([<=] n d) [T] ([If] ([?Div] d n) [F] ([Check] n ([+1] d)));
[IsPrime] : \n. [If] ([<=] n [1]) [F] ([Check] n [2]);
-- 测试
[IsPrime] [3]"],
    //
["数据结构", r"-- 构造对子 (a, b)
[Pair] : \abf. f a b;
-- 取左元素
[Fst]  : \p. p [T];
-- 取右元素
[Lst]  : \p. p [F];

-- 测试
[my_pair]: [Pair] [2] [3];
[*] ([Fst] [my_pair]) ([Lst] [my_pair])

"],
    //
["斐波那契数列",r"[Fib] : \n. [If] ([<=] n [1]) n ([+] ([Fib] ([-1] n)) ([Fib] ([-] n [2])));
[Fib] [3]"]
];

