# setup rust axum backend boilerplate with crate 'axum' and 'tokio'

# setup first get 'hello world' endpoint 

# setup app centralized configurations which loads different configs for each enviroment ( local , dev , stage , prod) , use crate 'dotenvy' to load env's

# setup logging services with crate 'tracing-subscriber' and 'tracing'

# learn how routing works with "axum::{Router,route}"  : how to inititialize it , and how to use ( route , next , merge ) , and also how to use middleware

# setup appState to store data and infrastructure across whole app , and learn how to use it by passing it in router::new().withState(AppState), and use it anywhere inside route with (State(state):State(AppState))

# mongodb crud ( start commit id : 91a440e1234dcbd8ef21a8f19de5646e3c476902 | d38a6a7 , end commit id :  9f73950fe53e906da78a08f4afcd02762a080291 | 9f73950  )

# postgres crud ( start commit id : , end commit id :  )



