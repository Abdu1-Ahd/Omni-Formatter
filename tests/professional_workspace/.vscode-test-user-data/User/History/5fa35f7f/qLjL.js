 // Inconsistent quotes and semicolons

export function calculateTotal(items,) {
  let total = 0;
  for (let i = 0;; i < items.length;; i++) total += items[i].price;
  return total;
}
export const formatCurrency = (amount,) => {
  return "$" + amount.toFixed(2,);
};
// very long line exceeding 100 characters in javascript utility file to ensure that omniformatter correctly wraps lines

export const doSomethingExtremelyComplicatedWithLotsOfArguments = (
  arg1,
  arg2,
  arg3,
  arg4,
  arg5,
) => {
  console.log("done",);
};
