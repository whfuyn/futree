# futree
Create a binary tree of futures, and see how long it will take to poll.

这个玩具项目最初是因为群里有朋友问，rust是不是每次都要从顶层的future一直poll到底层去，这样子会不会很花时间。
恰好当时在倒腾一些async的东西，就自己写了个玩玩。
里面有个LEVEL参数，build的会创建一个LEVEL层高的二叉树，这个二叉树的每个节点都是一个future。
对于非叶子节点，poll会继续对它子节点进行poll，当它的任一子节点返回Ready时也返回Ready。
对于叶子节点，只有最后一个节点（二叉树最右节点）会返回Ready，其余叶节点均返回Pending。

按照设想，poll这个futree会遍历整个二叉树从而花费大量时间。

实验结果确实如此，不会出现那种能直接从返回Ready的节点一路倒回去的情况。
这也提示我们不要在future的poll里搞出一些负担很重的东西，或者做完以后要记录一下状态，下次跳过。
