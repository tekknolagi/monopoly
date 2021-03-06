use std::convert::TryInto;
use std::error::Error;
use std::fmt;
// Derived from https://www.hasbro.com/common/instruct/00009.pdf

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerId(pub i8);

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: PlayerId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyId(i8);

#[derive(Clone, Debug, PartialEq)]
pub struct Money(i16);

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    name: &'static str,
    base: Money,
    houses: [Money; 4],
    hotel: Money,
    mortgage: Money,
    house_cost: Money,
    hotel_cost: (Money, i8), // ($cost, num_houses)
                             // TODO(emacs): double rent if player owns all lots on color?
}

#[derive(Clone, Debug, PartialEq)]
pub struct RollResult(pub i8, pub i8);

#[derive(Clone, Debug, PartialEq)]
pub struct ChanceCard;

#[derive(Clone, Debug, PartialEq)]
pub struct CommunityChestCard;

#[derive(Clone, Debug, PartialEq)]
pub enum Card {
    Chance(ChanceCard),
    CommunityChest(CommunityChestCard),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bid(PlayerId, Money);

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionType {
    BuyProperty(PlayerId, PropertyId),
    BuyGetOutOfJailFreeCard(PlayerId),
    SellProperty(PlayerId, PropertyId),
    PayRent(PlayerId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    ty: TransactionType,
    cost: Money,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    RollDice(PlayerId, RollResult),
    MoveForward(PlayerId, i8),
    BuyProperty(PlayerId, PropertyId),  // from the bank
    SellProperty(PlayerId, PropertyId), // to the bank
    BuyHouse(PlayerId, PropertyId),     // from the bank
    SellHouse(PlayerId, PropertyId),    // to the bank
    BuyHotel(PlayerId, PropertyId),     // from the bank
    SellHotel(PlayerId, PropertyId),    // to the bank
    PayTaxes(PlayerId, Money),          // to the bank
    ReceiveSalary(PlayerId),            // passing GO
    DrawCard(PlayerId, Card),
    GoToJail(PlayerId),
    PayJailFine(PlayerId),
    AuctionProperty(PropertyId, Vec<Bid>),
    MortgageProperty(PlayerId, PropertyId),
    UnmortgageProperty(PlayerId, PropertyId),
    TransactWithPlayer(PlayerId, Transaction),
    DeclareBankruptcy(PlayerId),
}

#[derive(Clone, Debug)]
enum Square {
    Go,
    Property(Property),
}

static SQUARES: &'static [Square] = &[
    Square::Go,
    Square::Property(Property {
        name: "Mediterranean Ave",
        base: Money(2),
        houses: [Money(10), Money(30), Money(90), Money(160)],
        hotel: Money(250),
        mortgage: Money(30),
        house_cost: Money(50),
        hotel_cost: (Money(50), 4),
    }),
];

#[derive(Clone)]
pub struct GameState {
    squares: Vec<Square>,
    players: Vec<Player>,
    events: Vec<Action>,
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameState")
            .field("players", &self.players)
            .field("events", &self.events)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateError {
    message: String,
}

impl StateError {
    pub fn new(message: &str) -> Self {
        StateError {
            message: message.to_string(),
        }
    }
}

impl Error for StateError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl GameState {
    pub fn init() -> GameState {
        GameState {
            squares: SQUARES.to_vec(),
            players: Vec::new(),
            events: Vec::new(),
        }
    }

    fn ensure_player(&self, player_id: i8) -> Result<(), StateError> {
        if player_id >= self.players.len().try_into().unwrap() {
            Err(StateError::new(
                format!("player {:?} is not a valid player", player_id).as_str(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn apply(&mut self, action: Action) -> Result<(), StateError> {
        match action {
            Action::RollDice(PlayerId(id), RollResult(one, two)) => {
                self.ensure_player(id)?;
                println!("player {:?} rolled {:?}", id, one + two);
                self.events.push(action);
                Ok(())
            }
            _ => Err(StateError::new("foo")),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn roll_dice_with_invalid_player_raises() {
        let mut state = GameState::init();
        let result = state.apply(Action::RollDice(PlayerId(0), RollResult(1, 2)));
        assert_eq!(
            result,
            Err(StateError::new("player 0 is not a valid player"))
        );
    }

    #[test]
    fn roll_dice_with_valid_player_logs_roll() {
        let mut state = GameState::init();
        let id = PlayerId(0);
        state.players.push(Player { id });
        let result = state.apply(Action::RollDice(id, RollResult(1, 2)));
        assert_eq!(result, Ok(()));
        assert_eq!(state.events, [Action::RollDice(id, RollResult(1, 2))]);
    }
}

// Mediterranean Avenue	Old Kent Road	60	2	10	30	90	160	250
// Baltic Avenue	Whitechapel Road	80	4	20	60	180	320	450
// Reading RR	Kings Cross Station	200	25 if 1 owned, 50 if 2 owned, 100 if 3 owned, 200 if all 4 owned
// Oriental Avenue	The Angel Islington	100	6	30	90	270	400	550
// Vermont Avenue	Euston Road	100	6	30	90	270	400	550
// Connecticut Avenue	Pentonville Road	120	8	40	100	300	450	600
// St. Charles Place	Pall Mall	140	10	50	150	450	625	750
// Electric Company	150	4×dice if 1 owned, 10×dice if both owned
// States Avenue	Whitehall	140	10	50	150	450	625	750
// Virginia Avenue	Northumberland Avenue	160	12	60	180	500	700	900
// Pennsylvania RR	Marylebone Station	200	25 if 1 owned, 50 if 2 owned, 100 if 3 owned, 200 if all 4 owned
// St. James Place	Bow Street	180	14	70	200	550	750	950
// Tennessee Avenue	Marlborough Street	180	14	70	200	550	750	950
// New York Avenue	Vine Street	200	16	80	220	600	800	1000
// Kentucky Avenue	The Strand	220	18	90	250	700	875	1050
// Indiana Avenue	Fleet Street	220	18	90	250	700	875	1050
// Illinois Avenue	Trafalgar Square	240	20	100	300	750	925	1100
// B&O RR	Fenchurch St Station	200	25 if 1 owned, 50 if 2 owned, 100 if 3 owned, 200 if all 4 owned
// Atlantic Avenue	Leicester Square	260	22	110	330	800	975	1150
// Ventnor Avenue	Coventry Street	260	22	110	330	800	975	1150
// Water Works	150	4×dice if 1 owned, 10×dice if both owned
// Marvin Gardens	Piccadilly	280	24	120	360	850	1025	1200
// Pacific Avenue	Regent Street	300	26	130	390	900	1100	1275
// North Carolina Avenue	Oxford Street	300	26	130	390	900	1100	1275
// Pennsylvania Avenue	Bond Street	320	28	150	450	1000	1200	1400
// Short Line	Liverpool Street Station	200	25 if 1 owned, 50 if 2 owned, 100 if 3 owned, 200 if all 4 owned
// Park Place	Park Lane	350	35	175	500	1100	1300	1500
// Boardwalk	Mayfair	400	50	200	600	1400	1700	2000
