using System;
using System.Linq;
using System.Collections.Generic;

namespace ConsoleApplication
{
    class DwarfAction
    {
        public static DwarfAction Nothing = new DwarfAction(); // Just for testing
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
            _dwarfs.Add(new Dwarf());
            _dwarfs.Add(new Dwarf());
        }
        
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
        }

        public IReadOnlyList<Area> Areas { get { return _areas; } }

        public bool InProgress { get { return _turn < 1; } }
        public int Turn { get { return _turn; } }
        


        public void EndTurn() { _turn++; }

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
                        if (false) {
                            var options = new DwarfAction[] { DwarfAction.Nothing };
                            area.Player.PlaceDwarf(options);
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
