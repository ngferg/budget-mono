// Maps an HTTP status code from the budget API to a short, user-facing message.
export function httpErrorMessage(status) {
  switch (status) {
    case 400:
      return "That request wasn't valid. Please check your input and try again.";
    case 401:
      return "Your session has expired. Please log in again.";
    case 402:
      return "Your free trial has ended. Subscribe for $5/month to keep using your budget.";
    case 403:
      return "You don't have permission to do that.";
    case 404:
      return "We couldn't find that — it may have already been removed.";
    case 409:
      return "That conflicts with something that already exists.";
    case 429:
      return "Too many requests. Please wait a moment and try again.";
    default:
      if (status >= 500) return "The server ran into a problem. Please try again shortly.";
      return `Something went wrong (error ${status}).`;
  }
}
