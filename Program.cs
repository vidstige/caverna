using System;
using System.Linq;
using System.Collections.Generic;

namespace ConsoleApplication
{
    class DwarfAction
    {
        private readonly string _name;
        public DwarfAction(string name)
        {
            _name = name;
        }
        public string Name { get { return _name; } }
        public static DwarfAction Nothing = new DwarfAction("Nothing 1"); // Just for testing
        public static DwarfAction Nothing2 = new DwarfAction("Nothing 2"); // Just for testing
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
            _available.Add(DwarfAction.Nothing);
            _available.Add(DwarfAction.Nothing2);
        }

        public IReadOnlyList<Area> Areas { get { return _areas; } }

        public bool InProgress { get { return _turn < 1; } }
        public int Turn { get { return _turn; } }

        public void EndTurn() { _turn++; }

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
