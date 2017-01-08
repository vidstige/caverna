using System;
using System.Linq;
using System.Collections.Generic;

namespace ConsoleApplication
{
    class Token
    {
           public static Token Stone = new Token();
           public static Token Wood = new Token();
           public static Token Food = new Token();
           public static Token Coal = new Token();
           public static Token Gold = new Token();
    }

    class DwarfAction
    {
        private readonly string _name;
        private readonly Dictionary<Token, int> _initial;
        private readonly Dictionary<Token, int> _replenish;
        
        public DwarfAction(string name, Dictionary<Token, int> initial, Dictionary<Token, int> replenish)
        {
            _name = name;
            _initial = initial;
            _replenish = replenish;
        }
        public string Name { get { return _name; } }

        public static List<DwarfAction> ForPlayers(int n) 
        {
            switch (n)
            {
                case 2:
                    return new List<DwarfAction> {
                        DriftMining(),
                        Logging,
                        WoodGathering,
                        Excavation,
                        Supplies,
                        Clearing,
                        StartingPlayer,
                        OreMining,
                        Sustenance
                    };
            }
            throw new Exception("Bad nmumber of players");
        }
        public static DwarfAction DriftMining() {
            return new DwarfAction("Drift Mining", new Dictionary<Token, int>(), new Dictionary<Token, int>{{Token.Stone, 1}});
        }

        public static DwarfAction Logging = new DwarfAction("Logging", null, null);
        public static DwarfAction WoodGathering = new DwarfAction("Wood Gathering", null, null);
        public static DwarfAction Excavation = new DwarfAction("Excavation", null, null);
        public static DwarfAction Supplies = new DwarfAction("Supplies", null, null);
        public static DwarfAction Clearing = new DwarfAction("Clearing", null, null);
        public static DwarfAction StartingPlayer = new DwarfAction("StartingPlayer", null, null);
        public static DwarfAction OreMining = new DwarfAction("OreMining", null, null);
        public static DwarfAction Sustenance = new DwarfAction("Sustenance", null, null);
    }

    interface IPlayer
    {
        string Name { get; }
        DwarfAction PlaceDwarf(IEnumerable<DwarfAction> options);
    }

    class RandomPlayer: IPlayer
    {
        private readonly string _name;

        public RandomPlayer(string name)
        {
            _name = name;
        }

        public string Name { get { return _name;} }

        public DwarfAction PlaceDwarf(IEnumerable<DwarfAction> options)
        {
            return options.First();
        }

    }

    class Dwarf
    {

    }
    
    // Holds the play area state for one player.
    class Area
    {
        private List<Dwarf> _dwarfs = new List<Dwarf>();
        private IPlayer _player;

        public Area(IPlayer player)
        {
            _player = player;
            //_dwarfs.Add(new Dwarf());
            _dwarfs.Add(new Dwarf());
        }
        
        public IReadOnlyList<Dwarf> Dwarfs { get { return _dwarfs; } }
        public IPlayer Player { get { return _player; } }
    }


    class Game
    {
        private int _turn = 0;
        private List<Area> _areas;
        private List<DwarfAction> _available = new List<DwarfAction>();
        private Dictionary<DwarfAction, Dwarf> _occupiedBy = new Dictionary<DwarfAction, Dwarf>();

        public Game(IEnumerable<IPlayer> players)
        {
            _areas = players.Select(p => new Area(p)).ToList();
            _available = DwarfAction.ForPlayers(players.Count());
        }

        public IReadOnlyList<Area> Areas { get { return _areas; } }

        public bool InProgress { get { return _turn < 2; } }
        public int Turn { get { return _turn; } }

        public void EndTurn()
        {
            // Return dwarf tokens to play Areas
            _occupiedBy.Clear();

            _turn++;
        }

        public Dwarf NextDwarf(Area area)
        {
            return area.Dwarfs.Where(dwarf => !_occupiedBy.Values.Contains(dwarf)).FirstOrDefault();
        }

        public int Score(Area area)
        {
            return 0;
        }

        public void Run()
        {            
            while (InProgress)
            {
                Console.WriteLine("Starting turn " + Turn);
                // Replenish

                // Action phase
                bool didPlace;
                do
                {
                    didPlace = false;
                    foreach (var area in Areas)
                    {
                        var dwarf = NextDwarf(area);
                        if (dwarf != null) {
                            var options = _available.Where(a => !_occupiedBy.ContainsKey(a));
                            var action =  area.Player.PlaceDwarf(options);
                            if (!options.Contains(action))
                            {
                                throw new Exception("Player made an illegal move.");
                            }
                            Console.WriteLine("{0} placing dwarf on {1}", area.Player.Name, action.Name);
                            _occupiedBy[action] = dwarf;
                            didPlace = true;
                        }
                    }
                } while (didPlace);
                
                // Harvest phase

                EndTurn();
            }   
        }
    }

    public class Program
    {
        public static void Main(string[] args)
        {
            var game = new Game(new IPlayer[] {new RandomPlayer("X"), new RandomPlayer("Y") });
            game.Run();

            // Score
            foreach (var area in game.Areas)
            {
                Console.WriteLine("Player {0}: {1}", area.Player.Name, game.Score(area));
            }
        }
    }
}
